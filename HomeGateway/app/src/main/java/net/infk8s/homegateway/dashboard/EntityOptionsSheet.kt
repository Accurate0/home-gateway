package net.infk8s.homegateway.dashboard

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.graphql.type.GarageDoorState

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EntityOptionsSheet(
    entity: EntityUi,
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

        is EntityUi.MediaPlayer -> ActionRow {
            ActionButton(if (entity.playing) "Pause" else "Play") { controls.mediaPlayPause(entity.id) }
            ActionButton("Stop") { controls.mediaStop(entity.id) }
        }

        is EntityUi.RobotVacuum -> ActionRow {
            ActionButton("Start") { controls.vacuumStart(entity.id) }
            ActionButton("Stop") { controls.vacuumStop(entity.id) }
            ActionButton("Dock") { controls.vacuumDock(entity.id) }
        }

        is EntityUi.EinkDisplay -> ActionRow {
            ActionButton("Refresh") { controls.takeScreenshot(entity.id) }
        }

        is EntityUi.Environment -> MetricGrid(
            listOfNotNull(
                entity.temperature?.let { Metric("Temperature", "%.1f°C".format(it)) },
                entity.humidity?.let { Metric("Humidity", "%.0f%%".format(it)) },
                entity.pressure?.let { Metric("Pressure", "%.0f hPa".format(it)) },
                entity.lux?.let { Metric("Illuminance", "%.0f lx".format(it)) },
                entity.uvIndex?.let { Metric("UV index", "%.1f".format(it)) },
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
