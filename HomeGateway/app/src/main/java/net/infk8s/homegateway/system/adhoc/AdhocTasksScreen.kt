package net.infk8s.homegateway.system.adhoc

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
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import java.time.Instant
import net.infk8s.homegateway.ui.OutlinedPill
import net.infk8s.homegateway.ui.Panel
import net.infk8s.homegateway.ui.relative
import net.infk8s.homegateway.ui.theme.StateTone

@Composable
fun AdhocTasksScreen(
    state: AdhocTasksUiState,
    running: String?,
    runError: String?,
    onRunCron: (AdhocCronTaskUi) -> Unit,
    onRunPending: () -> Unit,
    modifier: Modifier = Modifier,
) {
    when (state) {
        is AdhocTasksUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is AdhocTasksUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is AdhocTasksUiState.Loaded -> LazyColumn(
            modifier = modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            if (runError != null) {
                item(key = "error") {
                    Text(runError, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error)
                }
            }

            section("Scheduled", "cron") {}

            if (state.cron.isEmpty()) {
                item(key = "cron:empty") { EmptyText("No cron tasks registered.") }
            }

            items(state.cron, key = { "cron:${it.id}" }) { task ->
                CronTaskCard(task, running = running == task.name, enabled = running == null) { onRunCron(task) }
            }

            section("One-shot", "tasks") {
                OutlinedButton(onClick = onRunPending, enabled = running == null) {
                    Text(if (running == AdhocTasksViewModel.PENDING_KEY) "Running…" else "Run pending")
                }
            }

            if (state.tasks.isEmpty()) {
                item(key = "tasks:empty") { EmptyText("No one-shot tasks registered.") }
            }

            items(state.tasks, key = { "task:${it.id}" }) { task ->
                TaskCard(task)
            }
        }
    }
}

private fun LazyListScope.section(title: String, key: String, action: @Composable () -> Unit) {
    item(key = "section:$key") {
        Row(
            Modifier.fillMaxWidth().padding(top = 12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                title.uppercase(),
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.SemiBold,
                letterSpacing = 1.5.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            action()
        }
    }
}

@Composable
private fun EmptyText(text: String) {
    Text(text, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
}

@Composable
private fun CronTaskCard(task: AdhocCronTaskUi, running: Boolean, enabled: Boolean, onRun: () -> Unit) {
    Panel(Modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                TaskTitle(task.name)

                Text(
                    "${task.schedule} · next ${task.nextRunAt.relativeOrNever()}",
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )

                FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    OutcomePill(task.outcome)
                    task.flag?.let { OutlinedPill(it) }
                }

                Text(
                    buildList {
                        add("last ${task.lastRunAt.relativeOrNever()}")
                        task.durationMs?.let { add("${it}ms") }
                        task.rowsAffected?.let { add("$it rows") }
                    }.joinToString(" · "),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            OutlinedButton(onClick = onRun, enabled = enabled) {
                Text(if (running) "Running…" else "Run now")
            }
        }
    }
}

@Composable
private fun TaskCard(task: AdhocTaskUi) {
    Panel(Modifier.fillMaxWidth()) {
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            TaskTitle(task.name)

            Text(
                "#${task.ordinal}",
                style = MaterialTheme.typography.bodySmall,
                fontFamily = FontFamily.Monospace,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                if (task.pending) {
                    TonePill("pending", StateTone.LIGHT)
                } else {
                    TonePill("completed", StateTone.PRESENT)
                }
                if (task.checksumDrifted) {
                    TonePill("source changed", StateTone.OPEN)
                }
                task.flag?.let { OutlinedPill(it) }
            }

            task.completedAt?.let { completedAt ->
                Text(
                    buildList {
                        add("completed ${completedAt.relative()}")
                        task.durationMs?.let { add("${it}ms") }
                    }.joinToString(" · "),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun TaskTitle(name: String) {
    Text(name, style = MaterialTheme.typography.titleSmall, maxLines = 1, overflow = TextOverflow.Ellipsis)
}

@Composable
private fun OutcomePill(outcome: String?) {
    when (outcome) {
        null -> OutlinedPill("never run")
        "success" -> TonePill(outcome, StateTone.PRESENT)
        "error" -> OutlinedPill(
            outcome,
            color = MaterialTheme.colorScheme.error,
            borderColor = MaterialTheme.colorScheme.error.copy(alpha = 0.4f),
        )
        "held" -> TonePill(outcome, StateTone.LIGHT)
        else -> OutlinedPill(outcome)
    }
}

@Composable
private fun TonePill(text: String, tone: StateTone) {
    OutlinedPill(text, color = tone.accent, borderColor = tone.accent.copy(alpha = 0.4f))
}

private fun Instant?.relativeOrNever(): String = this?.relative() ?: "never"
