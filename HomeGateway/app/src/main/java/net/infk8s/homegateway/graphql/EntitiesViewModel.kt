package net.infk8s.homegateway.graphql

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.apollographql.apollo.api.Mutation
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.infk8s.homegateway.EntityControls
import net.infk8s.homegateway.graphql.type.Capability
import net.infk8s.homegateway.graphql.type.EntityCategory

/// A single entity rendered in the list, with just the state we display.
sealed interface EntityUi {
    val id: String
    val name: String
    val room: String?
    val category: EntityCategory
    val key: String

    // Live-state fields are nullable: the API returns null (plus a field error)
    // when the backing actor can't be reached, rather than dropping the entity.
    data class Light(
        override val id: String,
        override val name: String,
        override val room: String?,
        override val category: EntityCategory,
        val on: Boolean?,
        val dimmable: Boolean,
        val tunable: Boolean,
    ) : EntityUi {
        override val key get() = "light:$id"
    }

    data class Door(
        override val id: String,
        override val name: String,
        override val room: String?,
        override val category: EntityCategory,
        val open: Boolean?,
    ) : EntityUi {
        override val key get() = "door:$id"
    }

    data class Presence(
        override val id: String,
        override val name: String,
        override val room: String?,
        override val category: EntityCategory,
        val present: Boolean?,
    ) : EntityUi {
        override val key get() = "presence:$id"
    }

    data class Environment(
        override val id: String,
        override val name: String,
        override val room: String?,
        override val category: EntityCategory,
        val temperature: Double?,
        val humidity: Double?,
        val pressure: Double?,
        val lux: Double?,
        val uvIndex: Double?,
    ) : EntityUi {
        override val key get() = "environment:$id"
    }

    data class EinkDisplay(
        override val id: String,
        override val name: String,
        override val room: String?,
        override val category: EntityCategory,
        val batteryPercentage: Double?,
        val isCharging: Boolean?,
    ) : EntityUi {
        override val key get() = "eink:$id"
    }

    data class RobotVacuum(
        override val id: String,
        override val name: String,
        override val room: String?,
        override val category: EntityCategory,
        val status: String?,
        val batteryPercentage: Double?,
        val currentRoom: String?,
    ) : EntityUi {
        override val key get() = "vacuum:$id"
    }

    data class MediaPlayer(
        override val id: String,
        override val name: String,
        override val room: String?,
        override val category: EntityCategory,
        val playing: Boolean,
        val appName: String?,
        val mediaTitle: String?,
        val mediaSeriesTitle: String?,
    ) : EntityUi {
        override val key get() = "media:$id"
    }
}

/// One dashboard section, in the display order the backend chose.
data class EntitySectionUi(val title: String, val items: List<EntityUi>)

sealed interface EntitiesUiState {
    data object Loading : EntitiesUiState
    data class Error(val message: String) : EntitiesUiState
    data class Loaded(val sections: List<EntitySectionUi>) : EntitiesUiState
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

        val items = data.entities.mapNotNull { it.toUi() }
        // Section order and titles are a backend concern: render whatever it returns,
        // then append any category it didn't describe so nothing silently disappears.
        val titles = data.entitySections.associate { it.category to it.title }
        val ordered = data.entitySections.map { it.category } +
            items.map { it.category }.filterNot { titles.containsKey(it) }

        _state.value = EntitiesUiState.Loaded(ordered.distinct().toSections(items, titles))
    }

    private fun List<EntityCategory>.toSections(
        items: List<EntityUi>,
        titles: Map<EntityCategory, String>,
    ): List<EntitySectionUi> = mapNotNull { category ->
        val inSection = items
            .filter { it.category == category }
            .sortedBy { it.name.lowercase() }

        if (inSection.isEmpty()) null
        else EntitySectionUi(titles[category] ?: category.rawValue.lowercase(), inSection)
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

        val updated = current.sections.map { section ->
            section.copy(items = section.items.map { it.applyEvent(event) })
        }
        _state.value = EntitiesUiState.Loaded(updated)
    }

    private fun EntityUi.applyEvent(event: EventsSubscription.Events): EntityUi = when {
        event.onLightUpdate != null && this is EntityUi.Light && id == event.onLightUpdate.id ->
            copy(on = event.onLightUpdate.on)

        event.onDoorUpdate != null && this is EntityUi.Door && id == event.onDoorUpdate.id ->
            copy(open = event.onDoorUpdate.open)

        event.onPresenceUpdate != null && this is EntityUi.Presence && id == event.onPresenceUpdate.id ->
            copy(present = event.onPresenceUpdate.present)

        event.onEnvironmentUpdate != null && this is EntityUi.Environment && id == event.onEnvironmentUpdate.id ->
            applyReadings(event.onEnvironmentUpdate.readings)

        event.onMediaPlayerUpdate != null && this is EntityUi.MediaPlayer && id == event.onMediaPlayerUpdate.id ->
            copy(
                playing = event.onMediaPlayerUpdate.state in PLAYING_STATES,
                appName = event.onMediaPlayerUpdate.appName,
                mediaTitle = event.onMediaPlayerUpdate.mediaTitle,
                mediaSeriesTitle = event.onMediaPlayerUpdate.mediaSeriesTitle,
            )

        else -> this
    }

    fun controls() = EntityControls(
        setLight = { id, on -> mutate(if (on) LightOnMutation(id) else LightOffMutation(id)) },
        setBrightness = { id, value -> mutate(LightSetBrightnessMutation(id, value)) },
        setColourTemperature = { id, value ->
            mutate(LightSetColourTemperatureMutation(id, value))
        },
        mediaPlayPause = { mutate(MediaPlayPauseMutation(it)) },
        mediaStop = { mutate(MediaStopMutation(it)) },
        vacuumStart = { mutate(VacuumStartMutation(it)) },
        vacuumStop = { mutate(VacuumStopMutation(it)) },
        vacuumDock = { mutate(VacuumDockMutation(it)) },
        takeScreenshot = { mutate(EinkTakeScreenshotMutation(it)) },
    )

    /// Fire-and-forget: the resulting device state arrives over the subscription, so
    /// the mutation response itself is only interesting when it fails.
    private fun mutate(mutation: Mutation<*>) {
        viewModelScope.launch {
            try {
                val response = apollo.mutation(mutation).execute()
                if (response.hasErrors()) {
                    Log.w("EntitiesViewModel", "${mutation.name()} failed: ${response.errors}")
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w("EntitiesViewModel", "${mutation.name()} failed", e)
            }
        }
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
            EntityUi.Light(
                id = onLightEntity.id,
                name = onLightEntity.name,
                room = onLightEntity.room,
                category = onLightEntity.category,
                on = onLightEntity.on,
                dimmable = onLightEntity.capabilities.any { it == Capability.BRIGHTNESS },
                tunable = onLightEntity.capabilities.any { it == Capability.COLOUR_TEMP },
            )

        onDoorEntity != null ->
            EntityUi.Door(
                id = onDoorEntity.id,
                name = onDoorEntity.name,
                room = onDoorEntity.room,
                category = onDoorEntity.category,
                open = onDoorEntity.open,
            )

        onPresenceEntity != null ->
            EntityUi.Presence(
                id = onPresenceEntity.id,
                name = onPresenceEntity.name,
                room = onPresenceEntity.room,
                category = onPresenceEntity.category,
                present = onPresenceEntity.present,
            )

        onEnvironmentEntity != null ->
            EntityUi.Environment(
                id = onEnvironmentEntity.id,
                name = onEnvironmentEntity.name,
                room = onEnvironmentEntity.room,
                category = onEnvironmentEntity.category,
                temperature = onEnvironmentEntity.temperature,
                humidity = onEnvironmentEntity.humidity,
                pressure = onEnvironmentEntity.pressure,
                lux = onEnvironmentEntity.lux,
                uvIndex = onEnvironmentEntity.uvIndex,
            )

        onEinkDisplayEntity != null ->
            EntityUi.EinkDisplay(
                id = onEinkDisplayEntity.id,
                name = onEinkDisplayEntity.name,
                room = onEinkDisplayEntity.room,
                category = onEinkDisplayEntity.category,
                batteryPercentage = onEinkDisplayEntity.batteryPercentage,
                isCharging = onEinkDisplayEntity.isCharging,
            )

        onRobotVacuumEntity != null ->
            EntityUi.RobotVacuum(
                id = onRobotVacuumEntity.id,
                name = onRobotVacuumEntity.name,
                room = onRobotVacuumEntity.room,
                category = onRobotVacuumEntity.category,
                status = onRobotVacuumEntity.status,
                batteryPercentage = onRobotVacuumEntity.batteryPercentage,
                currentRoom = onRobotVacuumEntity.currentRoom,
            )

        onMediaPlayerEntity != null ->
            EntityUi.MediaPlayer(
                id = onMediaPlayerEntity.id,
                name = onMediaPlayerEntity.name,
                room = onMediaPlayerEntity.room,
                category = onMediaPlayerEntity.category,
                playing = onMediaPlayerEntity.playing,
                appName = onMediaPlayerEntity.appName,
                mediaTitle = onMediaPlayerEntity.mediaTitle,
                mediaSeriesTitle = onMediaPlayerEntity.mediaSeriesTitle,
            )

        else -> null
    }

    private companion object {
        val PLAYING_STATES = setOf("started", "resumed", "playing")
        const val INITIAL_BACKOFF_MS = 1_000L
        const val MAX_BACKOFF_MS = 30_000L
    }
}
