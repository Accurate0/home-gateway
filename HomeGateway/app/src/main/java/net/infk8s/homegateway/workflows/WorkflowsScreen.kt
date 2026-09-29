package net.infk8s.homegateway.workflows

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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.infk8s.homegateway.ui.OutlinedPill
import net.infk8s.homegateway.ui.Panel
import net.infk8s.homegateway.ui.theme.StateTone

@Composable
fun WorkflowsScreen(
    state: WorkflowsUiState,
    dryRunning: Boolean,
    dryRunError: String?,
    onToggle: (WorkflowUi, Boolean) -> Unit,
    onDryRun: (WorkflowUi) -> Unit,
    onHistory: (WorkflowUi) -> Unit,
    modifier: Modifier = Modifier,
) {
    when (state) {
        is WorkflowsUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is WorkflowsUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is WorkflowsUiState.Loaded -> LazyColumn(
            modifier = modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            item(key = "summary") {
                Text(
                    "${state.enabled} of ${state.total} enabled",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            if (dryRunError != null) {
                item(key = "error") {
                    Text(dryRunError, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error)
                }
            }

            state.groups.forEach { group ->
                item(key = "group:${group.name}") {
                    Text(
                        group.name.uppercase(),
                        style = MaterialTheme.typography.labelSmall,
                        fontWeight = FontWeight.SemiBold,
                        letterSpacing = 1.5.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(top = 12.dp),
                    )
                }

                items(group.workflows, key = { it.slug }) { workflow ->
                    WorkflowCard(
                        workflow,
                        dryRunning,
                        onToggle = { onToggle(workflow, it) },
                        onDryRun = { onDryRun(workflow) },
                        onHistory = { onHistory(workflow) },
                    )
                }
            }
        }
    }
}

@Composable
private fun WorkflowCard(
    workflow: WorkflowUi,
    dryRunning: Boolean,
    onToggle: (Boolean) -> Unit,
    onDryRun: () -> Unit,
    onHistory: () -> Unit,
) {
    Panel(Modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(
                    workflow.name,
                    style = MaterialTheme.typography.titleSmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    workflow.slug,
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )

                if (workflow.dryRun || workflow.reusable || workflow.modes.isNotEmpty()) {
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        if (workflow.dryRun) {
                            OutlinedPill("dry run")
                        }
                        if (workflow.reusable) {
                            OutlinedPill("reusable")
                        }
                        workflow.modes.forEach { mode ->
                            OutlinedPill(
                                mode,
                                color = StateTone.INFO.accent,
                                borderColor = StateTone.INFO.accent.copy(alpha = 0.4f),
                            )
                        }
                    }
                }
            }

            Switch(checked = workflow.enabled, onCheckedChange = onToggle)
        }

        Row(
            Modifier.fillMaxWidth().padding(top = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            OutlinedButton(onClick = onDryRun, enabled = !dryRunning, modifier = Modifier.weight(1f)) {
                Text("Dry run")
            }
            OutlinedButton(onClick = onHistory, modifier = Modifier.weight(1f)) {
                Text("History")
            }
        }
    }
}
