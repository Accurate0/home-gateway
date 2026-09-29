package net.infk8s.homegateway.modes

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import java.time.Instant
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.ModesQuery
import net.infk8s.homegateway.graphql.SetModeMutation
import net.infk8s.homegateway.graphql.type.LightTarget
import net.infk8s.homegateway.graphql.type.Mode
import net.infk8s.homegateway.ui.parseInstant

data class VacationActionUi(val at: Instant?, val on: Boolean, val slot: Int, val onFraction: Double)

data class VacationLightUi(
    val address: String,
    val deviceId: String?,
    val name: String,
    val coverage: Int,
    val currentTarget: LightTarget,
    val actions: List<VacationActionUi>,
)

data class VacationUi(
    val enabled: Boolean,
    val window: String,
    val jitter: String,
    val minObservations: Int,
    val lights: List<VacationLightUi>,
)

sealed interface ModesUiState {
    data object Loading : ModesUiState
    data class Error(val message: String) : ModesUiState
    data class Loaded(val active: Mode, val vacation: VacationUi?) : ModesUiState
}

class ModesViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val _state = MutableStateFlow<ModesUiState>(ModesUiState.Loading)
    val state: StateFlow<ModesUiState> = _state.asStateFlow()

    fun refresh() {
        viewModelScope.launch { load() }
    }

    fun select(mode: Mode) {
        val current = _state.value as? ModesUiState.Loaded ?: return
        if (current.active == mode) {
            return
        }

        _state.value = current.copy(active = mode)

        viewModelScope.launch {
            try {
                val response = apollo.mutation(SetModeMutation(mode)).execute()
                if (response.hasErrors()) {
                    Log.w(TAG, "setMode($mode) returned errors: ${response.errors}")
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w(TAG, "setMode($mode) failed", e)
            }

            delay(REFRESH_DELAY_MS)
            load()
        }
    }

    private suspend fun load() {
        try {
            val response = apollo.query(ModesQuery()).execute()
            val data = response.data
                ?: throw IllegalStateException(response.errors?.firstOrNull()?.message ?: "Failed to load modes")

            _state.value = ModesUiState.Loaded(data.mode.active, data.mode.node.onVacationMode?.toUi())
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Log.w(TAG, "failed to load modes", e)

            if (_state.value !is ModesUiState.Loaded) {
                _state.value = ModesUiState.Error(e.message ?: "Failed to load modes")
            }
        }
    }

    private fun ModesQuery.OnVacationMode.toUi() = VacationUi(
        enabled = enabled,
        window = window,
        jitter = jitter,
        minObservations = minObservations,
        lights = lights.map { light ->
            VacationLightUi(
                address = light.address,
                deviceId = light.deviceId,
                name = light.name,
                coverage = light.coverage,
                currentTarget = light.currentTarget,
                actions = light.actions.map { VacationActionUi(parseInstant(it.at), it.on, it.slot, it.onFraction) },
            )
        },
    )

    private companion object {
        const val TAG = "ModesViewModel"
        const val REFRESH_DELAY_MS = 500L
    }
}
