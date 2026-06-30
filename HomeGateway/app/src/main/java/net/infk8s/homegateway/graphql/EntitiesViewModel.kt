package net.infk8s.homegateway.graphql

import android.util.Log
import androidx.lifecycle.ViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/// A single entity rendered in the list, with just the state we display.
sealed interface EntityUi {
    val id: String
    val name: String

    // Live-state fields are nullable: the API returns null (plus a field error)
    // when the backing actor can't be reached, rather than dropping the entity.
    data class Light(override val id: String, override val name: String, val on: Boolean?) : EntityUi
    data class Door(override val id: String, override val name: String, val open: Boolean?) : EntityUi
    data class Presence(override val id: String, override val name: String, val present: Boolean?) : EntityUi
    data class Environment(
        override val id: String,
        override val name: String,
        val temperature: Double?,
        val humidity: Double?,
        val pressure: Double?,
        val lux: Double?,
        val uvIndex: Double?,
    ) : EntityUi
}

sealed interface EntitiesUiState {
    data object Loading : EntitiesUiState
    data class Error(val message: String) : EntitiesUiState
    data class Loaded(val items: List<EntityUi>) : EntitiesUiState
}

class EntitiesViewModel : ViewModel() {
    private val apollo = ApolloProvider.client

    private val _state = MutableStateFlow<EntitiesUiState>(EntitiesUiState.Loading)
    val state: StateFlow<EntitiesUiState> = _state.asStateFlow()

    /// Drives the live connection for as long as the caller's coroutine is active.
    /// The Activity launches this from a STARTED-scoped lifecycle, so the WebSocket
    /// only runs while the UI is visible — when the phone locks or the app is
    /// backgrounded the coroutine is cancelled and the socket torn down, instead of
    /// leaving Apollo to spin reconnect attempts against a Doze-suspended network.
    suspend fun run() {
        var backoffMs = INITIAL_BACKOFF_MS
        while (true) {
            try {
                // Re-fetch a full snapshot on every (re)connect: the subscription only
                // carries deltas, so any events missed while disconnected would otherwise
                // leave the UI permanently stale. Querying first reconciles that.
                loadSnapshot()
                backoffMs = INITIAL_BACKOFF_MS
                subscribe()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w("EntitiesViewModel", "live connection lost, reconnecting in ${backoffMs}ms", e)
                // Keep showing the last snapshot while reconnecting; only surface an
                // error if we never managed to load anything in the first place.
                if (_state.value !is EntitiesUiState.Loaded) {
                    _state.value = EntitiesUiState.Error(e.message ?: "Failed to load entities")
                }
                delay(backoffMs)
                backoffMs = (backoffMs * 2).coerceAtMost(MAX_BACKOFF_MS)
            }
        }
    }

    private suspend fun loadSnapshot() {
        val response = apollo.query(EntitiesQuery()).execute()
        val data = response.data
            ?: throw IllegalStateException(
                response.errors?.firstOrNull()?.message ?: "Failed to load entities",
            )
        // Partial errors (e.g. an unreachable actor) only null individual fields,
        // not the entity — keep rendering the list and just log them.
        if (response.hasErrors()) {
            Log.w("EntitiesViewModel", "entities query returned field errors: ${response.errors}")
        }

        val items = data.entities.mapNotNull { it.toUi() }.sortedBy { it.name.lowercase() }
        _state.value = EntitiesUiState.Loaded(items)
    }

    private suspend fun subscribe() {
        // A dropped WebSocket throws out of collect; the run() loop catches it, backs
        // off, and reconnects with a fresh snapshot rather than resuming stale deltas.
        apollo.subscription(EventsSubscription()).toFlow()
            .collect { response ->
                val event = response.data?.events ?: return@collect
                applyEvent(event)
            }
    }

    private fun applyEvent(event: EventsSubscription.Events) {
        val current = _state.value as? EntitiesUiState.Loaded ?: return

        val updated = current.items.map { item ->
            when {
                event.onLightUpdate != null && item is EntityUi.Light && item.id == event.onLightUpdate.id ->
                    item.copy(on = event.onLightUpdate.on)

                event.onDoorUpdate != null && item is EntityUi.Door && item.id == event.onDoorUpdate.id ->
                    item.copy(open = event.onDoorUpdate.open)

                event.onPresenceUpdate != null && item is EntityUi.Presence && item.id == event.onPresenceUpdate.id ->
                    item.copy(present = event.onPresenceUpdate.present)

                event.onEnvironmentUpdate != null && item is EntityUi.Environment && item.id == event.onEnvironmentUpdate.id ->
                    item.applyReadings(event.onEnvironmentUpdate.readings)

                else -> item
            }
        }
        _state.value = EntitiesUiState.Loaded(updated)
    }

    private fun EntityUi.Environment.applyReadings(
        readings: List<EventsSubscription.Reading>,
    ): EntityUi.Environment {
        val byMetric = readings.associate { it.metric to it.value }
        return copy(
            temperature = byMetric["temperature"] ?: temperature,
            humidity = byMetric["humidity"] ?: humidity,
            pressure = byMetric["pressure"] ?: pressure,
            lux = byMetric["lux"] ?: lux,
            uvIndex = byMetric["uv_index"] ?: byMetric["uvIndex"] ?: uvIndex,
        )
    }

    private fun EntitiesQuery.Entity.toUi(): EntityUi? = when {
        onLightEntity != null ->
            EntityUi.Light(onLightEntity.id, onLightEntity.name, onLightEntity.on)

        onDoorEntity != null ->
            EntityUi.Door(onDoorEntity.id, onDoorEntity.name, onDoorEntity.open)

        onPresenceEntity != null ->
            EntityUi.Presence(onPresenceEntity.id, onPresenceEntity.name, onPresenceEntity.present)

        onEnvironmentEntity != null ->
            EntityUi.Environment(
                id = onEnvironmentEntity.id,
                name = onEnvironmentEntity.name,
                temperature = onEnvironmentEntity.temperature,
                humidity = onEnvironmentEntity.humidity,
                pressure = onEnvironmentEntity.pressure,
                lux = onEnvironmentEntity.lux,
                uvIndex = onEnvironmentEntity.uvIndex,
            )

        else -> null
    }

    private companion object {
        const val INITIAL_BACKOFF_MS = 1_000L
        const val MAX_BACKOFF_MS = 30_000L
    }
}
