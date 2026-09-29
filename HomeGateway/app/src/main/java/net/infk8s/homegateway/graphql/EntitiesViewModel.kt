package net.infk8s.homegateway.graphql

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.apollographql.apollo.api.Mutation
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlin.math.roundToInt
import java.time.Instant
import java.util.UUID
import net.infk8s.homegateway.dashboard.CustomSection
import net.infk8s.homegateway.dashboard.CustomTile
import net.infk8s.homegateway.dashboard.DashboardLayout
import net.infk8s.homegateway.dashboard.DashboardMode
import net.infk8s.homegateway.dashboard.EinkConfigUi
import net.infk8s.homegateway.dashboard.EntityControls
import net.infk8s.homegateway.dashboard.LayoutSection
import net.infk8s.homegateway.dashboard.LayoutTile
import net.infk8s.homegateway.dashboard.LightLevels
import net.infk8s.homegateway.dashboard.move
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.quicktiles.QuickTiles
import net.infk8s.homegateway.widget.EntityWidgets
import net.infk8s.homegateway.graphql.type.EntityCategory
import net.infk8s.homegateway.graphql.type.GarageDoorState

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
        val colour: Boolean,
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

    data class GarageDoor(
        override val id: String,
        override val name: String,
        override val room: String?,
        override val category: EntityCategory,
        val state: GarageDoorState?,
        val batteryPercentage: Double?,
    ) : EntityUi {
        override val key get() = "garage_door:$id"
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
        val pm25: Int?,
        val vocIndex: Int?,
        val lastSeen: Instant?,
    ) : EntityUi {
        override val key get() = "environment:$id"
    }

    data class Plant(
        override val id: String,
        override val name: String,
        override val room: String?,
        override val category: EntityCategory,
        val soilMoisture: Double?,
        val batteryPercentage: Double?,
    ) : EntityUi {
        override val key get() = "plant:$id"
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

data class CategoryUi(val category: EntityCategory, val title: String)

data class EntitySectionUi(
    val key: String,
    val title: String,
    val items: List<EntityUi>,
    val editable: Boolean,
)

sealed interface EntitiesUiState {
    data object Loading : EntitiesUiState
    data class Error(val message: String) : EntitiesUiState
    data class Loaded(
        val mode: DashboardMode,
        val sections: List<EntitySectionUi>,
        val offline: Set<String>,
    ) : EntitiesUiState
}

private sealed interface EntitiesSnapshot {
    data object Loading : EntitiesSnapshot
    data class Error(val message: String) : EntitiesSnapshot
    data class Loaded(val entities: List<EntityUi>, val categories: List<CategoryUi>) : EntitiesSnapshot
}

class EntitiesViewModel(application: Application) : AndroidViewModel(application) {
    private val apollo = application.gateway.apollo

    private val dashboard = application.gateway.database.dashboard()

    private val preferences = application.gateway.dashboardPreferences

    private val source = application.gateway.entities

    private var surfaceRefresh: Job? = null

    private val snapshot = MutableStateFlow<EntitiesSnapshot>(EntitiesSnapshot.Loading)

    private val layout = MutableStateFlow<DashboardLayout?>(null)

    private val offline = MutableStateFlow<Set<String>>(emptySet())

    private val _commandFailures = MutableSharedFlow<String>(extraBufferCapacity = COMMAND_FAILURE_BUFFER)
    val commandFailures: SharedFlow<String> = _commandFailures.asSharedFlow()

    val lightLevels = LightLevels()

    val state: StateFlow<EntitiesUiState> = combine(snapshot, layout, offline) { snapshot, layout, offline ->
        when {
            layout == null -> EntitiesUiState.Loading
            snapshot is EntitiesSnapshot.Loaded ->
                EntitiesUiState.Loaded(layout.mode, layout.arrange(snapshot.entities, snapshot.categories), offline)
            snapshot is EntitiesSnapshot.Error -> EntitiesUiState.Error(snapshot.message)
            else -> EntitiesUiState.Loading
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, EntitiesUiState.Loading)

    init {
        viewModelScope.launch {
            layout.value = DashboardLayout.from(
                preferences.mode,
                dashboard.sections(),
                dashboard.tiles(),
                dashboard.customSections(),
                dashboard.customTiles(),
            )
        }
    }

    fun setMode(mode: DashboardMode) {
        val current = layout.value ?: return
        if (current.mode == mode) {
            return
        }

        preferences.mode = mode
        if (mode != DashboardMode.CUSTOM || current.customSections.isNotEmpty()) {
            layout.value = current.copy(mode = mode)
            return
        }

        val source = arranged(current).orEmpty()
        val sections = source.mapIndexed { index, section ->
            CustomSection(UUID.randomUUID().toString(), section.title, index)
        }
        val tiles = source.zip(sections).flatMap { (from, section) ->
            from.items.mapIndexed { index, entity -> CustomTile(entity.key, section.id, index) }
        }

        layout.value = current.copy(mode = mode).withCustomSections(sections).withCustomTiles(tiles, emptyList())
        viewModelScope.launch { dashboard.seedCustom(sections, tiles) }
    }

    fun moveTile(from: String, to: String) {
        val current = layout.value ?: return
        val sections = arranged(current) ?: return

        val flat = sections
            .flatMap { listOf(DashboardLayout.headerKey(it.key)) + it.items.map(EntityUi::key) }
            .toMutableList()
        if (!flat.move(from, to)) {
            return
        }

        val regrouped = regroup(flat) ?: return
        val before = sections.associate { section -> section.key to section.items.map(EntityUi::key) }
        val changed = regrouped.filter { (section, keys) -> before[section] != keys }
        if (changed.isEmpty()) {
            return
        }

        if (current.mode == DashboardMode.CUSTOM) {
            placeCustomTiles(current, changed)
        } else {
            reorderTiles(current, before, changed)
        }
    }

    fun moveSection(from: String, to: String) {
        val current = layout.value ?: return
        val sections = arranged(current) ?: return

        val order = sections
            .map { it.key }
            .filterNot { current.mode == DashboardMode.CUSTOM && it == DashboardLayout.UNSORTED_KEY }
            .toMutableList()
        if (!order.move(from, to)) {
            return
        }

        if (current.mode == DashboardMode.CUSTOM) {
            val byId = current.customSections.associateBy { it.id }
            val updated = order.mapIndexedNotNull { index, id -> byId[id]?.copy(position = index) }

            layout.value = current.withCustomSections(updated)
            viewModelScope.launch { dashboard.upsertCustomSections(updated) }
        } else {
            layout.value = current.withSectionOrder(order)
            viewModelScope.launch {
                dashboard.upsertSections(order.mapIndexed { index, key -> LayoutSection(current.mode, key, index) })
            }
        }
    }

    fun addSection(title: String) {
        val current = layout.value ?: return
        val position = (current.customSections.maxOfOrNull { it.position } ?: -1) + 1
        val section = CustomSection(UUID.randomUUID().toString(), title.trim(), position)

        layout.value = current.withCustomSections(current.customSections + section)
        viewModelScope.launch { dashboard.upsertCustomSections(listOf(section)) }
    }

    fun renameSection(id: String, title: String) {
        val current = layout.value ?: return
        val section = current.customSections.firstOrNull { it.id == id }?.copy(title = title.trim()) ?: return

        layout.value = current.withCustomSections(current.customSections.map { if (it.id == id) section else it })
        viewModelScope.launch { dashboard.upsertCustomSections(listOf(section)) }
    }

    fun deleteSection(id: String) {
        val current = layout.value ?: return

        layout.value = current.withCustomSections(current.customSections.filterNot { it.id == id })
        viewModelScope.launch { dashboard.deleteCustomSection(id) }
    }

    private fun arranged(layout: DashboardLayout): List<EntitySectionUi>? =
        (snapshot.value as? EntitiesSnapshot.Loaded)?.let { layout.arrange(it.entities, it.categories) }

    private fun regroup(flat: List<String>): Map<String, List<String>>? {
        val groups = LinkedHashMap<String, MutableList<String>>()
        var section: MutableList<String>? = null

        for (key in flat) {
            if (DashboardLayout.isHeaderKey(key)) {
                section = mutableListOf()
                groups[DashboardLayout.sectionOfHeader(key)] = section
            } else {
                section ?: return null
                section.add(key)
            }
        }

        return groups
    }

    private fun reorderTiles(
        current: DashboardLayout,
        before: Map<String, List<String>>,
        changed: Map<String, List<String>>,
    ) {
        val crossesSections = changed.any { (section, keys) -> keys.toSet() != before[section].orEmpty().toSet() }
        if (crossesSections) {
            return
        }

        val order = changed.values.flatten()

        layout.value = current.withTileOrder(order)
        viewModelScope.launch {
            dashboard.upsertTiles(order.mapIndexed { index, key -> LayoutTile(current.mode, key, index) })
        }
    }

    private fun placeCustomTiles(current: DashboardLayout, changed: Map<String, List<String>>) {
        val placed = changed
            .filterKeys { it != DashboardLayout.UNSORTED_KEY }
            .flatMap { (section, keys) -> keys.mapIndexed { index, key -> CustomTile(key, section, index) } }
        val unsorted = changed[DashboardLayout.UNSORTED_KEY].orEmpty()

        layout.value = current.withCustomTiles(placed, unsorted).withTileOrder(unsorted)
        viewModelScope.launch {
            dashboard.placeCustomTiles(
                placed,
                unsorted,
                unsorted.mapIndexed { index, key -> LayoutTile(DashboardMode.CUSTOM, key, index) },
            )
        }
    }

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
                if (snapshot.value !is EntitiesSnapshot.Loaded) {
                    snapshot.value = EntitiesSnapshot.Error(e.message ?: "Failed to load entities")
                }
                delay(backoffMs)
                backoffMs = (backoffMs * 2).coerceAtMost(MAX_BACKOFF_MS)
            }
        }
    }

    private suspend fun loadSnapshot() {
        val fetched = source.fetch()

        snapshot.value = EntitiesSnapshot.Loaded(fetched.entities, fetched.categories)
        refreshSurfaces()
    }

    private fun refreshSurfaces() {
        if (surfaceRefresh?.isActive == true) {
            return
        }

        surfaceRefresh = viewModelScope.launch {
            delay(SURFACE_REFRESH_DELAY_MS)
            EntityWidgets.refreshAll(getApplication())
            QuickTiles.refreshAll(getApplication())
        }
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
        event.onDeviceConnectionUpdate?.let { update ->
            offline.value = if (update.connected) offline.value - update.deviceId else offline.value + update.deviceId
            return
        }

        val current = snapshot.value as? EntitiesSnapshot.Loaded ?: return

        event.onCommandFailedUpdate?.let { failure ->
            val name = current.entities.firstOrNull { it.id == failure.id }?.name ?: failure.id
            _commandFailures.tryEmit("$name didn't respond to a ${failure.kind} command after ${failure.attempts} attempts")
            return
        }

        val updated = current.entities.map { it.applyEvent(event) }
        if (updated == current.entities) {
            return
        }

        snapshot.value = current.copy(entities = updated)
        source.update(EntitySnapshot(updated, current.categories))
        refreshSurfaces()
    }

    private fun EntityUi.applyEvent(event: EventsSubscription.Events): EntityUi = when {
        event.onLightUpdate != null && this is EntityUi.Light && id == event.onLightUpdate.id ->
            copy(on = event.onLightUpdate.on)

        event.onDoorUpdate != null && this is EntityUi.Door && id == event.onDoorUpdate.id ->
            copy(open = event.onDoorUpdate.open)

        event.onGarageDoorUpdate != null && this is EntityUi.GarageDoor && id == event.onGarageDoorUpdate.id ->
            copy(state = event.onGarageDoorUpdate.garageState)

        event.onPresenceUpdate != null && this is EntityUi.Presence && id == event.onPresenceUpdate.id ->
            copy(present = event.onPresenceUpdate.present)

        event.onEnvironmentUpdate != null && this is EntityUi.Environment && id == event.onEnvironmentUpdate.id ->
            applyReadings(event.onEnvironmentUpdate.readings)

        event.onPlantUpdate != null && this is EntityUi.Plant && id == event.onPlantUpdate.id ->
            copy(soilMoisture = event.onPlantUpdate.soilMoisture)

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
        run = { id, command -> mutate(command.mutation(id)) },
        setLight = { id, on -> mutate(if (on) LightOnMutation(id) else LightOffMutation(id)) },
        setBrightness = { id, value -> mutate(LightSetBrightnessMutation(id, value)) },
        setColourTemperature = { id, value ->
            mutate(LightSetColourTemperatureMutation(id, value))
        },
        colourTemperatureMove = { id, value -> mutate(LightColourTemperatureMoveMutation(id, value)) },
        setColour = { id, hex -> mutate(LightSetColourMutation(id, hex)) },
        mediaPlayPause = { mutate(MediaPlayPauseMutation(it)) },
        mediaStop = { mutate(MediaStopMutation(it)) },
        vacuumStart = { mutate(VacuumStartMutation(it)) },
        vacuumStop = { mutate(VacuumStopMutation(it)) },
        vacuumDock = { mutate(VacuumDockMutation(it)) },
        garageDoorOpen = { mutate(GarageDoorOpenMutation(it)) },
        garageDoorClose = { mutate(GarageDoorCloseMutation(it)) },
        takeScreenshot = { mutate(EinkTakeScreenshotMutation(it)) },
        loadEinkConfig = ::loadEinkConfig,
    )

    private suspend fun loadEinkConfig(id: String): EinkConfigUi? = try {
        val response = apollo.query(EinkConfigQuery(id)).execute()
        if (response.hasErrors()) {
            Log.w("EntitiesViewModel", "einkConfig($id) returned errors: ${response.errors}")
        }

        response.data?.einkDisplay?.deviceConfig?.let { config ->
            EinkConfigUi(config.refreshIntervalMins, config.imageUrl, config.clearScreen)
        }
    } catch (e: CancellationException) {
        throw e
    } catch (e: Exception) {
        Log.w("EntitiesViewModel", "einkConfig($id) failed", e)
        null
    }

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
            pm25 = byMetric["pm25"]?.roundToInt() ?: pm25,
            vocIndex = byMetric["voc_index"]?.roundToInt() ?: vocIndex,
            lastSeen = Instant.now(),
        )
    }

    private companion object {
        val PLAYING_STATES = setOf("started", "resumed", "playing")
        const val INITIAL_BACKOFF_MS = 1_000L
        const val MAX_BACKOFF_MS = 30_000L
        const val SURFACE_REFRESH_DELAY_MS = 3_000L
        const val COMMAND_FAILURE_BUFFER = 8
    }
}
