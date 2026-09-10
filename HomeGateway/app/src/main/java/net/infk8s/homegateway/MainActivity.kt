package net.infk8s.homegateway

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import com.google.firebase.messaging.FirebaseMessaging
import kotlin.concurrent.thread
import kotlinx.coroutines.launch
import net.infk8s.homegateway.graphql.EntitiesUiState
import net.infk8s.homegateway.graphql.EntitiesViewModel
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.graphql.type.NotificationInteractionKind
import net.infk8s.homegateway.notifications.NotificationInteractions
import net.infk8s.homegateway.notifications.PushPayload
import net.infk8s.homegateway.ui.theme.HomeGatewayTheme

class MainActivity : ComponentActivity() {
    private val requestNotificationPermission =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { /* no-op */ }

    private val entitiesViewModel: EntitiesViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (savedInstanceState == null) {
            reportNotificationOpened(intent)
        }
        ensureNotificationPermission()
        registerPushToken()
        enableEdgeToEdge()
        setContent {
            HomeGatewayTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    val state by entitiesViewModel.state.collectAsStateWithLifecycle()
                    // Run the live WebSocket only while the UI is at least STARTED.
                    // repeatOnLifecycle cancels run() when the app is backgrounded or the
                    // phone locks, and restarts it on return — which re-fetches a fresh
                    // snapshot so the list is correct after time away.
                    val lifecycleOwner = LocalLifecycleOwner.current
                    LaunchedEffect(lifecycleOwner) {
                        lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
                            entitiesViewModel.run()
                        }
                    }
                    EntitiesScreen(state, entitiesViewModel.controls(), Modifier.padding(innerPadding))
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        reportNotificationOpened(intent)
    }

    private fun reportNotificationOpened(intent: Intent?) {
        val notificationId = intent?.getStringExtra(PushPayload.KEY_NOTIFICATION_ID) ?: return
        intent.removeExtra(PushPayload.KEY_NOTIFICATION_ID)

        lifecycleScope.launch {
            NotificationInteractions.record(notificationId, NotificationInteractionKind.OPENED)
        }
    }

    private fun ensureNotificationPermission() {
        val granted = ContextCompat.checkSelfPermission(
            this,
            Manifest.permission.POST_NOTIFICATIONS,
        ) == PackageManager.PERMISSION_GRANTED
        if (!granted) {
            requestNotificationPermission.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }

    private fun registerPushToken() {
        FirebaseMessaging.getInstance().token.addOnCompleteListener { task ->
            if (!task.isSuccessful) {
                Log.e("MainActivity", "failed to fetch fcm token", task.exception)
                return@addOnCompleteListener
            }
            val token = task.result
            // Listener runs on the main thread; register off it to avoid network-on-main.
            thread { PushTokenRegistrar.register(token) }
        }
    }
}

@Composable
fun EntitiesScreen(
    state: EntitiesUiState,
    controls: EntityControls,
    modifier: Modifier = Modifier,
) {
    when (state) {
        is EntitiesUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is EntitiesUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is EntitiesUiState.Loaded -> LazyColumn(
            modifier = modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            state.sections.forEach { section ->
                item(key = "section:${section.title}") {
                    Text(
                        section.title,
                        style = MaterialTheme.typography.titleMedium,
                        modifier = Modifier.padding(top = 8.dp, bottom = 4.dp),
                    )
                }
                items(section.items, key = { it.key }) { entity ->
                    EntityCard(entity, controls)
                }
            }
        }
    }
}

/// The mutations a card can invoke, passed down so the composables stay
/// previewable and free of a ViewModel dependency.
data class EntityControls(
    val setLight: (String, Boolean) -> Unit,
    val setBrightness: (String, Int) -> Unit,
    val setColourTemperature: (String, Int) -> Unit,
    val mediaPlayPause: (String) -> Unit,
    val mediaStop: (String) -> Unit,
    val vacuumStart: (String) -> Unit,
    val vacuumStop: (String) -> Unit,
    val vacuumDock: (String) -> Unit,
    val takeScreenshot: (String) -> Unit,
)

@Composable
fun EntityCard(entity: EntityUi, controls: EntityControls) {
    Card(Modifier.fillMaxWidth()) {
        Column(
            Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(Modifier.weight(1f)) {
                    Text(entity.name, style = MaterialTheme.typography.titleSmall)
                    Text(
                        listOfNotNull(entity.room, entity.details()).joinToString(" · "),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (entity is EntityUi.Light) {
                    Switch(
                        checked = entity.on == true,
                        onCheckedChange = { controls.setLight(entity.id, it) },
                    )
                }
            }

            EntityActions(entity, controls)
        }
    }
}

@Composable
private fun EntityActions(entity: EntityUi, controls: EntityControls) {
    when (entity) {
        is EntityUi.Light -> {
            if (entity.dimmable) {
                BrightnessBar(entity, controls)
            }
            if (entity.tunable) {
                ColourTemperatureBar(entity, controls)
            }
        }

        is EntityUi.MediaPlayer -> ActionRow {
            AssistChip(
                onClick = { controls.mediaPlayPause(entity.id) },
                label = { Text(if (entity.playing) "Pause" else "Play") },
            )
            AssistChip(
                onClick = { controls.mediaStop(entity.id) },
                label = { Text("Stop") },
            )
        }

        is EntityUi.RobotVacuum -> ActionRow {
            AssistChip(onClick = { controls.vacuumStart(entity.id) }, label = { Text("Start") })
            AssistChip(onClick = { controls.vacuumStop(entity.id) }, label = { Text("Stop") })
            AssistChip(onClick = { controls.vacuumDock(entity.id) }, label = { Text("Dock") })
        }

        is EntityUi.EinkDisplay -> ActionRow {
            AssistChip(
                onClick = { controls.takeScreenshot(entity.id) },
                label = { Text("Refresh") },
            )
        }

        is EntityUi.Door, is EntityUi.Presence, is EntityUi.Environment -> Unit
    }
}

/// The API is write-only for brightness — no level is reported back — so the bar
/// tracks the last value this device set, seeded at half like the web dashboard.
/// Commit on release only, so dragging doesn't flood the light with mqtt writes.
@Composable
private fun BrightnessBar(entity: EntityUi.Light, controls: EntityControls) {
    var brightness by rememberSaveable(entity.id) { mutableFloatStateOf(BRIGHTNESS_MAX / 2f) }

    Row(
        Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Slider(
            value = brightness,
            onValueChange = { brightness = it },
            onValueChangeFinished = {
                controls.setBrightness(entity.id, brightness.roundToInt())
            },
            valueRange = 0f..BRIGHTNESS_MAX,
            enabled = entity.on != false,
            modifier = Modifier.weight(1f),
        )
        Text(
            "%d%%".format((brightness / BRIGHTNESS_MAX * 100).roundToInt()),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

/// Mireds run backwards against how the bar reads, so the slider is inverted:
/// dragging right warms the light. Like brightness, the value is write-only, so
/// the bar tracks what this device last set rather than the light's real state.
@Composable
private fun ColourTemperatureBar(entity: EntityUi.Light, controls: EntityControls) {
    var mireds by rememberSaveable(entity.id) { mutableFloatStateOf(MIREDS_COOL) }

    Row(
        Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Slider(
            value = MIREDS_COOL + MIREDS_WARM - mireds,
            onValueChange = { mireds = MIREDS_COOL + MIREDS_WARM - it },
            onValueChangeFinished = {
                controls.setColourTemperature(entity.id, mireds.roundToInt())
            },
            valueRange = MIREDS_COOL..MIREDS_WARM,
            enabled = entity.on != false,
            modifier = Modifier.weight(1f),
        )
        Text(
            "%dK".format((1_000_000f / mireds).roundToInt()),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun ActionRow(content: @Composable RowScope.() -> Unit) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
        content = content,
    )
}

private fun EntityUi.details(): String = when (this) {
    is EntityUi.Light -> on?.let { if (it) "On" else "Off" } ?: "Unknown"
    is EntityUi.Door -> open?.let { if (it) "Open" else "Closed" } ?: "Unknown"
    is EntityUi.Presence -> present?.let { if (it) "Present" else "Away" } ?: "Unknown"
    is EntityUi.Environment -> buildList {
        temperature?.let { add("%.1f°C".format(it)) }
        humidity?.let { add("%.0f%%".format(it)) }
        pressure?.let { add("%.0f hPa".format(it)) }
        lux?.let { add("%.0f lx".format(it)) }
        uvIndex?.let { add("UV %.1f".format(it)) }
    }.joinToString(" · ").ifEmpty { "Unknown" }

    is EntityUi.EinkDisplay -> buildList {
        batteryPercentage?.let { add("%.0f%%".format(it)) }
        if (isCharging == true) add("Charging")
    }.joinToString(" · ").ifEmpty { "Unknown" }

    is EntityUi.RobotVacuum -> buildList {
        status?.let { add(it) }
        currentRoom?.let { add(it) }
        batteryPercentage?.let { add("%.0f%%".format(it)) }
    }.joinToString(" · ").ifEmpty { "Unknown" }

    is EntityUi.MediaPlayer -> buildList {
        add(if (playing) "Playing" else "Paused")
        appName?.let { add(it) }
        listOfNotNull(mediaSeriesTitle, mediaTitle).firstOrNull()?.let { add(it) }
    }.joinToString(" · ")
}

private const val MIREDS_COOL = 153f
private const val MIREDS_WARM = 500f
private const val BRIGHTNESS_MAX = 254f
