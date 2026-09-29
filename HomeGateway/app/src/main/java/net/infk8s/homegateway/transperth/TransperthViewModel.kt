package net.infk8s.homegateway.transperth

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.TransperthRoutesQuery
import net.infk8s.homegateway.graphql.TransperthUpdatesSubscription
import net.infk8s.homegateway.graphql.keepLive
import net.infk8s.homegateway.ui.parseInstant

class TransperthViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val _state = MutableStateFlow<TransperthUiState>(TransperthUiState.Loading)
    val state: StateFlow<TransperthUiState> = _state.asStateFlow()

    suspend fun run() {
        keepLive(TAG, ::load, ::listen) { e ->
            if (_state.value !is TransperthUiState.Loaded) {
                _state.value = TransperthUiState.Error(e.message ?: "Failed to load departures")
            }
        }
    }

    private suspend fun load() {
        val response = apollo.query(TransperthRoutesQuery()).execute()
        val data = response.data
            ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load departures")

        _state.value = TransperthUiState.Loaded(data.transperth.routes.map { it.toUi() })
    }

    private suspend fun listen() {
        apollo.subscription(TransperthUpdatesSubscription()).toFlow()
            .collect { response ->
                if (response.data?.events?.onTransperthUpdate != null) {
                    load()
                }
            }
    }

    private fun TransperthRoutesQuery.Route.toUi() = RouteUi(
        id = id,
        origin = origin,
        destination = destination,
        updatedAt = parseInstant(updatedAt),
        stale = stale,
        departures = departures.mapNotNull { departure ->
            val departsAt = parseInstant(departure.liveDeparture)
                ?: parseInstant(departure.scheduledDeparture)
                ?: return@mapNotNull null

            DepartureUi(
                line = departure.line,
                headsign = departure.headsign,
                platform = departure.platform,
                departsAt = departsAt,
                delayMinutes = departure.delayMinutes,
                live = departure.live,
            )
        },
    )

    private companion object {
        const val TAG = "TransperthViewModel"
    }
}
