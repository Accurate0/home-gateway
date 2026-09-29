package net.infk8s.homegateway.workflows

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.roundToInt
import net.infk8s.homegateway.R
import net.infk8s.homegateway.ui.OutlinedPill
import net.infk8s.homegateway.ui.Panel
import net.infk8s.homegateway.ui.relative
import net.infk8s.homegateway.ui.theme.StateTone

@Composable
fun RunsScreen(
    state: RunsUiState,
    traces: Map<String, RunTraceState>,
    filter: String?,
    pendingEventId: String?,
    onFilter: (String?) -> Unit,
    onExpand: (RunUi) -> Unit,
    modifier: Modifier = Modifier,
) {
    when (state) {
        is RunsUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is RunsUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is RunsUiState.Loaded -> RunsList(state.runs, traces, filter, pendingEventId, onFilter, onExpand, modifier)
    }
}

@Composable
private fun RunsList(
    runs: List<RunUi>,
    traces: Map<String, RunTraceState>,
    filter: String?,
    pendingEventId: String?,
    onFilter: (String?) -> Unit,
    onExpand: (RunUi) -> Unit,
    modifier: Modifier,
) {
    var toggled by rememberSaveable { mutableStateOf(emptyList<String>()) }

    val pendingRunId = pendingEventId?.let { eventId -> runs.firstOrNull { it.eventId == eventId }?.id }
    val slugs = runs.map { it.slug }.distinct().sorted()
    val visible = if (filter == null) runs else runs.filter { it.slug == filter }

    fun isExpanded(run: RunUi) = (run.id in toggled) != (run.id == pendingRunId)

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        item(key = "filters") {
            Row(
                Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                FilterChip(selected = filter == null, onClick = { onFilter(null) }, label = { Text("All") })
                (slugs + listOfNotNull(filter?.takeIf { it !in slugs })).forEach { slug ->
                    FilterChip(
                        selected = filter == slug,
                        onClick = { onFilter(slug) },
                        label = { Text(slug, fontFamily = FontFamily.Monospace) },
                    )
                }
            }
        }

        if (visible.isEmpty()) {
            item(key = "empty") {
                Text(
                    "No workflow runs recorded yet.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        items(visible, key = { it.id }) { run ->
            val expanded = isExpanded(run)
            LaunchedEffect(run.id, expanded) {
                if (expanded) {
                    onExpand(run)
                }
            }

            RunCard(
                run,
                expanded = expanded,
                pending = run.eventId == pendingEventId,
                trace = traces[run.id],
                onToggle = {
                    toggled = if (run.id in toggled) toggled - run.id else toggled + run.id
                },
            )
        }
    }
}

@Composable
private fun RunCard(
    run: RunUi,
    expanded: Boolean,
    pending: Boolean,
    trace: RunTraceState?,
    onToggle: () -> Unit,
) {
    val rotation by animateFloatAsState(if (expanded) 90f else 0f, label = "chevron")

    Panel(
        Modifier.fillMaxWidth(),
        borderColor = if (pending) StateTone.INFO.accent.copy(alpha = 0.6f) else MaterialTheme.colorScheme.outlineVariant,
    ) {
        Row(
            Modifier
                .fillMaxWidth()
                .clickable(onClick = onToggle),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.Top,
        ) {
            Icon(
                painterResource(R.drawable.ic_chevron_right),
                contentDescription = if (expanded) "Collapse" else "Expand",
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.size(20.dp).rotate(rotation),
            )

            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        run.name,
                        style = MaterialTheme.typography.titleSmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f, fill = false),
                    )
                    if (run.dryRun) {
                        OutlinedPill("dry run")
                    }
                }
                Text(
                    run.slug,
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                run.error?.let {
                    Text(
                        it,
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = FontFamily.Monospace,
                        color = MaterialTheme.colorScheme.error,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }

            Column(horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(4.dp)) {
                OutcomePill(run.outcome)
                Text(
                    listOfNotNull(run.startedAt?.relative(), "${run.durationMs}ms").joinToString(" · "),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        if (expanded) {
            HorizontalDivider(Modifier.padding(vertical = 12.dp), color = MaterialTheme.colorScheme.outlineVariant)
            RunTrace(trace)
        }
    }
}

@Composable
private fun OutcomePill(outcome: String) {
    val color = when (outcome) {
        "success" -> StateTone.PRESENT.accent
        "error" -> MaterialTheme.colorScheme.error
        else -> null
    }

    if (color == null) {
        OutlinedPill(outcome)
    } else {
        OutlinedPill(outcome, color = color, borderColor = color.copy(alpha = 0.4f))
    }
}

@Composable
private fun RunTrace(trace: RunTraceState?) {
    when (trace) {
        null, RunTraceState.Loading -> TraceNote("Loading trace…")
        RunTraceState.Missing -> TraceNote("Trace not found.")
        is RunTraceState.Error -> TraceNote(trace.message, MaterialTheme.colorScheme.error)
        is RunTraceState.Loaded -> {
            if (trace.trace.steps.isEmpty()) {
                TraceNote("No steps recorded.")
            } else {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    trace.trace.steps.forEach { RunStep(it) }
                }
            }

            trace.trace.trigger?.let { TriggerVariables(it) }
        }
    }
}

@Composable
private fun RunStep(step: RunStepUi) {
    val skipped = step.outcome == "guard_skipped"

    Row(
        Modifier.padding(start = (step.depth * 20).dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Box(
            Modifier
                .padding(top = 5.dp)
                .size(8.dp)
                .background(stepColor(step.outcome), CircleShape),
        )

        Column(Modifier.weight(1f)) {
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.Center,
            ) {
                Text(
                    step.kind,
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    color = if (skipped) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface,
                    textDecoration = if (skipped) TextDecoration.LineThrough else null,
                )
                Text(
                    stepLabel(step.outcome).uppercase(),
                    style = MaterialTheme.typography.labelSmall,
                    fontSize = 10.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                if (!skipped) {
                    Text(
                        formatMicros(step.durationUs),
                        style = MaterialTheme.typography.labelSmall,
                        fontSize = 10.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            step.detail?.let { TraceLine(it, maxLines = 1) }
            step.guard?.let { TraceLine("when $it") }
            step.error?.let { TraceLine(it, color = MaterialTheme.colorScheme.error) }
        }
    }
}

@Composable
private fun TriggerVariables(trigger: String) {
    var open by rememberSaveable { mutableStateOf(false) }

    TextButton(onClick = { open = !open }, contentPadding = PaddingValues(0.dp)) {
        Text(
            if (open) "Hide trigger variables" else "Trigger variables",
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }

    if (open) {
        Text(
            trigger,
            style = MaterialTheme.typography.bodySmall,
            fontFamily = FontFamily.Monospace,
            modifier = Modifier
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.surfaceVariant, RoundedCornerShape(8.dp))
                .horizontalScroll(rememberScrollState())
                .padding(12.dp),
        )
    }
}

@Composable
private fun TraceLine(text: String, color: Color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines: Int = Int.MAX_VALUE) {
    Text(
        text,
        style = MaterialTheme.typography.bodySmall,
        fontFamily = FontFamily.Monospace,
        color = color,
        maxLines = maxLines,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
private fun TraceNote(text: String, color: Color = MaterialTheme.colorScheme.onSurfaceVariant) {
    Text(text, style = MaterialTheme.typography.bodySmall, color = color)
}

@Composable
private fun stepColor(outcome: String): Color = when (outcome) {
    "ran" -> StateTone.PRESENT.accent
    "dry_run" -> StateTone.INFO.accent
    "error" -> MaterialTheme.colorScheme.error
    "running" -> StateTone.LIGHT.accent
    "guard_skipped" -> MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)
    else -> MaterialTheme.colorScheme.onSurfaceVariant
}

private fun stepLabel(outcome: String): String = when (outcome) {
    "dry_run" -> "dry run"
    "guard_skipped" -> "skipped"
    "running" -> "unfinished"
    else -> outcome
}

private fun formatMicros(us: Int): String = when {
    us < 1_000 -> "${us}µs"
    us < 10_000 -> "%.1fms".format(us / 1_000.0)
    us < 1_000_000 -> "${(us / 1_000.0).roundToInt()}ms"
    else -> "%.2fs".format(us / 1_000_000.0)
}
