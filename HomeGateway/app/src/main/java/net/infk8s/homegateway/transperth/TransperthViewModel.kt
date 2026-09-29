package net.infk8s.homegateway.transperth

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.TransperthRoutesQuery
import net.infk8s.homegateway.ui.parseInstant

class TransperthViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val _state = MutableStateFlow<TransperthUiState>(TransperthUiState.Loading)
    val state: StateFlow<TransperthUiState> = _state.asStateFlow()

    suspend fun run() {
        while (true) {
            load()
            delay(REFRESH_INTERVAL_MS)
        }
    }

    private suspend fun load() {
        try {
            val response = apollo.query(TransperthRoutesQuery()).execute()
            val data = response.data
                ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load departures")

            _state.value = TransperthUiState.Loaded(data.transperth.routes.map { it.toUi() })
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Log.w(TAG, "failed to load departures", e)

            if (_state.value !is TransperthUiState.Loaded) {
                _state.value = TransperthUiState.Error(e.message ?: "Failed to load departures")
            }
        }
    }

    private fun TransperthRoutesQuery.Route.toUi() = RouteUi(
        id = id,
        origin = origin,
        destination = destination,
        updatedAt = parseInstant(updatedAt),
        stale = stale,
        departures = departures.map { departure ->
            DepartureUi(
                line = departure.line,
                headsign = departure.headsign,
                platform = departure.platform,
                departsAt = parseInstant(departure.liveDeparture) ?: parseInstant(departure.scheduledDeparture),
                delayMinutes = departure.delayMinutes,
                minutesAway = departure.minutesAway,
                live = departure.live,
            )
        },
    )

    private companion object {
        const val TAG = "TransperthViewModel"
        const val REFRESH_INTERVAL_MS = 60_000L
    }
}
