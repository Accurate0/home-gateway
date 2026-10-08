package net.infk8s.homegateway.dashboard

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.graphql.type.AirPurifierMode
import net.infk8s.homegateway.graphql.type.GarageDoorState
import net.infk8s.homegateway.ui.relative

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EntityOptionsSheet(
    entity: EntityUi,
    offline: Boolean,
    controls: EntityControls,
    lightLevels: LightLevels,
    onDismiss: () -> Unit,
) {
    ModalBottomSheet(onDismissRequest = onDismiss) {
        Column(
            Modifier
                .fillMaxWidth()
                .padding(horizontal = 24.dp)
                .padding(bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(20.dp),
        ) {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(Modifier.weight(1f)) {
                    Text(entity.name, style = MaterialTheme.typography.titleMedium)
                    Text(
                        listOfNotNull(entity.room, entity.details(), "Offline".takeIf { offline }).joinToString(" · "),
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

                if (entity is EntityUi.AirPurifier) {
                    Switch(
                        checked = entity.on == true,
                        onCheckedChange = { controls.setAirPurifier(entity.id, it) },
                    )
                }
            }

            EntityOptions(entity, controls, lightLevels)
        }
    }
}

@Composable
private fun EntityOptions(entity: EntityUi, controls: EntityControls, lightLevels: LightLevels) {
    when (entity) {
        is EntityUi.Light -> {
            if (entity.dimmable) {
                BrightnessBar(entity, controls, lightLevels)
            }
            if (entity.tunable) {
                ColourTemperatureBar(entity, controls, lightLevels)
                ColourTemperatureMove(entity, controls)
            }
            if (entity.colour) {
                ColourSwatches(entity, controls)
            }
        }

        is EntityUi.GarageDoor -> ActionRow {
            ActionButton(
                "Open",
                enabled = entity.state != GarageDoorState.OPEN && entity.state != GarageDoorState.OPENING,
            ) { controls.garageDoorOpen(entity.id) }
            ActionButton(
                "Close",
                enabled = entity.state != GarageDoorState.CLOSED && entity.state != GarageDoorState.CLOSING,
            ) { controls.garageDoorClose(entity.id) }
        }

        is EntityUi.AirPurifier -> {
            ActionRow {
                AirPurifierMode.knownEntries.forEach { mode ->
                    ActionButton(
                        mode.rawValue.lowercase().replaceFirstChar { it.uppercase() },
                        enabled = entity.on != true || entity.mode != mode,
                    ) { controls.setAirPurifierMode(entity.id, mode) }
                }
            }

            ActionRow {
                AIR_PURIFIER_SPEEDS.forEach { speed ->
                    ActionButton(
                        "Speed $speed",
                        enabled = entity.on != true ||
                            entity.mode != AirPurifierMode.MANUAL ||
                            entity.speed != speed,
                    ) { controls.setAirPurifierSpeed(entity.id, speed) }
                }
            }

            LabelledRow("Display", null) {
                Switch(
                    checked = entity.displayOn == true,
                    enabled = entity.displayOn != null,
                    onCheckedChange = { controls.setAirPurifierDisplay(entity.id, it) },
                )
            }

            MetricGrid(
                listOfNotNull(
                    entity.pm25?.let { Metric("PM2.5", "%.0f µg/m³".format(it)) },
                    entity.aqi?.let { Metric("AQI", "%.0f".format(it)) },
                    entity.cadr?.let { Metric("CADR", "%.0f m³/h".format(it)) },
                    entity.filterLife?.let { Metric("Filter life", "%.0f%%".format(it)) },
                ),
            )
        }

        is EntityUi.MediaPlayer -> ActionRow {
            ActionButton(if (entity.playing) "Pause" else "Play") { controls.mediaPlayPause(entity.id) }
            ActionButton("Stop") { controls.mediaStop(entity.id) }
        }

        is EntityUi.RobotVacuum -> ActionRow {
            ActionButton("Start") { controls.vacuumStart(entity.id) }
            ActionButton("Stop") { controls.vacuumStop(entity.id) }
            ActionButton("Dock") { controls.vacuumDock(entity.id) }
        }

        is EntityUi.EinkDisplay -> {
            ActionRow {
                ActionButton("Refresh") { controls.takeScreenshot(entity.id) }
            }

            EinkConfig(entity, controls)
        }

        is EntityUi.Environment -> MetricGrid(
            listOfNotNull(
                entity.temperature?.let { Metric("Temperature", "%.1f°C".format(it)) },
                entity.humidity?.let { Metric("Humidity", "%.0f%%".format(it)) },
                entity.pressure?.let { Metric("Pressure", "%.0f hPa".format(it)) },
                entity.lux?.let { Metric("Illuminance", "%.0f lx".format(it)) },
                entity.uvIndex?.let { Metric("UV index", "%.1f".format(it)) },
                entity.pm25?.let { Metric("PM2.5", "$it µg/m³") },
                entity.vocIndex?.let { Metric("VOC index", "$it") },
                entity.lastSeen?.let { Metric("Last seen", it.relative()) },
            ),
        )

        is EntityUi.Plant -> MetricGrid(
            listOfNotNull(
                entity.soilMoisture?.let { Metric("Soil moisture", "%.0f%%".format(it)) },
                entity.moistureStatus()?.let { Metric("Status", it) },
                entity.batteryPercentage?.let { Metric("Battery", "%.0f%%".format(it)) },
            ),
        )

        is EntityUi.Door, is EntityUi.Presence -> Unit
    }
}

@Composable
private fun MetricGrid(metrics: List<Metric>) {
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        metrics.chunked(2).forEach { row ->
            Row(Modifier.fillMaxWidth()) {
                row.forEach { metric ->
                    Column(Modifier.weight(1f)) {
                        Text(
                            metric.label.uppercase(),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Text(metric.value, style = MaterialTheme.typography.bodyLarge)
                    }
                }

                if (row.size == 1) {
                    Column(Modifier.weight(1f)) {}
                }
            }
        }
    }
}

/// The API is write-only for brightness — no level is reported back — so the bar
/// tracks the last value this device set, seeded at half like the web dashboard.
/// Commit on release only, so dragging doesn't flood the light with mqtt writes.
@Composable
private fun BrightnessBar(entity: EntityUi.Light, controls: EntityControls, lightLevels: LightLevels) {
    val brightness = lightLevels.brightness[entity.id] ?: (BRIGHTNESS_MAX / 2f)

    LabelledSlider("Brightness", "%d%%".format((brightness / BRIGHTNESS_MAX * 100).roundToInt())) {
        Slider(
            value = brightness,
            onValueChange = { lightLevels.brightness[entity.id] = it },
            onValueChangeFinished = { controls.setBrightness(entity.id, brightness.roundToInt()) },
            valueRange = 0f..BRIGHTNESS_MAX,
            enabled = entity.on != false,
        )
    }
}

/// Mireds run backwards against how the bar reads, so the slider is inverted:
/// dragging right warms the light. Like brightness, the value is write-only, so
/// the bar tracks what this device last set rather than the light's real state.
@Composable
private fun ColourTemperatureBar(entity: EntityUi.Light, controls: EntityControls, lightLevels: LightLevels) {
    val mireds = lightLevels.mireds[entity.id] ?: MIREDS_COOL

    LabelledSlider("Colour temperature", "%dK".format((1_000_000f / mireds).roundToInt())) {
        Slider(
            value = MIREDS_COOL + MIREDS_WARM - mireds,
            onValueChange = { lightLevels.mireds[entity.id] = MIREDS_COOL + MIREDS_WARM - it },
            onValueChangeFinished = { controls.setColourTemperature(entity.id, mireds.roundToInt()) },
            valueRange = MIREDS_COOL..MIREDS_WARM,
            enabled = entity.on != false,
        )
    }
}

@Composable
private fun ColourTemperatureMove(entity: EntityUi.Light, controls: EntityControls) {
    val enabled = entity.on != false

    LabelledRow("Shift colour temperature", "Hold to shift") {
        ActionRow {
            HoldButton("Warmer", enabled) { controls.colourTemperatureMove(entity.id, it * COLOUR_MOVE_RATE) }
            HoldButton("Cooler", enabled) { controls.colourTemperatureMove(entity.id, -it * COLOUR_MOVE_RATE) }
        }
    }
}

@Composable
private fun RowScope.HoldButton(label: String, enabled: Boolean, onMove: (Int) -> Unit) {
    val colour = MaterialTheme.colorScheme.onSurface.copy(alpha = if (enabled) 1f else DISABLED_ALPHA)

    Box(
        Modifier
            .weight(1f)
            .height(40.dp)
            .clip(CircleShape)
            .border(1.dp, MaterialTheme.colorScheme.outlineVariant, CircleShape)
            .pointerInput(enabled) {
                if (!enabled) {
                    return@pointerInput
                }

                detectTapGestures(
                    onPress = {
                        onMove(1)
                        tryAwaitRelease()
                        onMove(0)
                    },
                )
            },
        contentAlignment = Alignment.Center,
    ) {
        Text(label, style = MaterialTheme.typography.labelLarge, color = colour)
    }
}

@Composable
private fun ColourSwatches(entity: EntityUi.Light, controls: EntityControls) {
    LabelledRow("Colour", null) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            COLOUR_SWATCHES.forEach { hex ->
                Box(
                    Modifier
                        .size(32.dp)
                        .clip(CircleShape)
                        .background(Color(android.graphics.Color.parseColor(hex)))
                        .border(1.dp, MaterialTheme.colorScheme.outlineVariant, CircleShape)
                        .clickable(enabled = entity.on != false) { controls.setColour(entity.id, hex) },
                )
            }
        }
    }
}

@Composable
private fun EinkConfig(entity: EntityUi.EinkDisplay, controls: EntityControls) {
    var config by remember(entity.id) { mutableStateOf<EinkConfigUi?>(null) }

    LaunchedEffect(entity.id) {
        config = controls.loadEinkConfig(entity.id)
    }

    val loaded = config ?: return

    MetricGrid(
        listOfNotNull(
            loaded.refreshIntervalMins?.let { Metric("Refresh interval", "$it min") },
            loaded.clearScreen?.let { Metric("Clear screen", if (it) "Yes" else "No") },
            loaded.imageUrl?.let { Metric("Image", it) },
        ),
    )
}

@Composable
private fun LabelledRow(label: String, hint: String?, content: @Composable () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(
                label.uppercase(),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            if (hint != null) {
                Text(
                    hint,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        content()
    }
}

@Composable
private fun LabelledSlider(label: String, value: String, slider: @Composable () -> Unit) {
    Column {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(
                label.uppercase(),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                value,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        slider()
    }
}

@Composable
private fun ActionRow(content: @Composable RowScope.() -> Unit) {
    Row(
        Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        content = content,
    )
}

@Composable
private fun RowScope.ActionButton(label: String, enabled: Boolean = true, onClick: () -> Unit) {
    FilledTonalButton(onClick = onClick, enabled = enabled, modifier = Modifier.weight(1f)) {
        Text(label)
    }
}

private data class Metric(val label: String, val value: String)

private const val MIREDS_COOL = 153f
private const val MIREDS_WARM = 500f
private const val BRIGHTNESS_MAX = 254f
private const val COLOUR_MOVE_RATE = 40
private const val DISABLED_ALPHA = 0.38f

private val AIR_PURIFIER_SPEEDS = listOf(1, 2, 3)

private val COLOUR_SWATCHES = listOf(
    "#ff5a5a",
    "#ff9d3c",
    "#ffd23c",
    "#3ce17a",
    "#3cc7ff",
    "#6a7bff",
    "#c86bff",
    "#ffffff",
)
