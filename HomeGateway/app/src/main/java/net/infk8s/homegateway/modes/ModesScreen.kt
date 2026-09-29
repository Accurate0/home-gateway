package net.infk8s.homegateway.modes

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.infk8s.homegateway.graphql.type.LightTarget
import net.infk8s.homegateway.graphql.type.Mode
import net.infk8s.homegateway.ui.OutlinedPill
import net.infk8s.homegateway.ui.Panel
import net.infk8s.homegateway.ui.clock
import net.infk8s.homegateway.ui.relative
import net.infk8s.homegateway.ui.theme.StateTone

@Composable
fun ModesScreen(
    state: ModesUiState,
    onSelect: (Mode) -> Unit,
    modifier: Modifier = Modifier,
) {
    when (state) {
        is ModesUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is ModesUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is ModesUiState.Loaded -> LazyColumn(
            modifier = modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            item(key = "house") { HouseMode(state.active, onSelect) }

            state.vacation?.let { vacation ->
                item(key = "vacation") { VacationHeader(state.active, vacation) }
                items(vacation.lights, key = { it.address }) { VacationLight(it) }
            }
        }
    }
}

@Composable
private fun HouseMode(active: Mode, onSelect: (Mode) -> Unit) {
    Panel(Modifier.fillMaxWidth()) {
        Text("House mode", style = MaterialTheme.typography.titleSmall)
        Text(
            "Exactly one mode is active. Workflows limited to other modes will not fire.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(top = 4.dp, bottom = 12.dp),
        )

        SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
            MODES.forEachIndexed { index, mode ->
                SegmentedButton(
                    selected = active == mode,
                    onClick = { onSelect(mode) },
                    shape = SegmentedButtonDefaults.itemShape(index, MODES.size),
                    label = { Text(mode.label(), maxLines = 1) },
                )
            }
        }
    }
}

@Composable
private fun VacationHeader(active: Mode, vacation: VacationUi) {
    Column(Modifier.padding(top = 16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(
                "VACATION REPLAY",
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.SemiBold,
                letterSpacing = 1.5.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (active == Mode.VACATION) {
                OutlinedPill(
                    "replaying",
                    color = StateTone.LIGHT.accent,
                    borderColor = StateTone.LIGHT.accent.copy(alpha = 0.4f),
                )
            }
            if (!vacation.enabled) {
                OutlinedPill("replay off")
            }
        }

        Text(
            "Replays ${vacation.window} of light history, jittered by up to ${vacation.jitter}, for slots with " +
                "at least ${vacation.minObservations} observations. Plan for the next 24 hours:",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun VacationLight(light: VacationLightUi) {
    Panel(Modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text(light.name, style = MaterialTheme.typography.titleSmall, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(
                    light.deviceId ?: light.address,
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }

            TargetPill(light.currentTarget)
            OutlinedPill("${light.coverage}/$SLOTS_PER_DAY slots")
        }

        val note = when {
            light.coverage == 0 -> "Not enough history yet — this light is left alone."
            light.actions.isEmpty() -> "No switches planned in the next 24 hours."
            else -> null
        }

        if (note != null) {
            Text(
                note,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(top = 12.dp),
            )
        } else {
            FlowRow(
                Modifier.padding(top = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                light.actions.forEach { ActionChip(it) }
            }
        }
    }
}

@Composable
private fun TargetPill(target: LightTarget) {
    val label = when (target) {
        LightTarget.ON -> "on"
        LightTarget.OFF -> "off"
        LightTarget.LEAVE_ALONE -> "left alone"
        else -> target.rawValue.lowercase()
    }

    if (target == LightTarget.ON) {
        OutlinedPill("now $label", color = StateTone.LIGHT.accent, borderColor = StateTone.LIGHT.accent.copy(alpha = 0.4f))
    } else {
        OutlinedPill("now $label")
    }
}

@Composable
private fun ActionChip(action: VacationActionUi) {
    val color = if (action.on) StateTone.LIGHT.accent else MaterialTheme.colorScheme.onSurfaceVariant

    Surface(
        shape = CircleShape,
        color = Color.Transparent,
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
    ) {
        Row(
            Modifier.padding(horizontal = 12.dp, vertical = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(6.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(action.at?.clock() ?: "—", style = MaterialTheme.typography.labelMedium, color = color)
            Text(
                (if (action.on) "on" else "off").uppercase(),
                style = MaterialTheme.typography.labelSmall,
                fontSize = 10.sp,
                color = color,
            )
            action.at?.let {
                Text(
                    it.relative(),
                    style = MaterialTheme.typography.labelSmall,
                    fontSize = 10.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

private fun Mode.label(): String = rawValue.lowercase().replaceFirstChar { it.uppercase() }

private val MODES = listOf(Mode.HOME, Mode.AWAY, Mode.VACATION, Mode.GUEST)

private const val SLOTS_PER_DAY = 48
