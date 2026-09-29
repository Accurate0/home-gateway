package net.infk8s.homegateway.jellyfin

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
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.infk8s.homegateway.ui.OutlinedPill
import net.infk8s.homegateway.ui.Panel
import net.infk8s.homegateway.ui.theme.StateTone

@Composable
fun JellyfinScreen(state: JellyfinUiState, modifier: Modifier = Modifier) {
    when (state) {
        is JellyfinUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is JellyfinUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is JellyfinUiState.Loaded -> if (state.sessions.isEmpty()) {
            Box(modifier.fillMaxSize(), Alignment.Center) {
                Text(
                    "Nothing playing",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        } else {
            LazyColumn(
                modifier = modifier.fillMaxSize(),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                items(state.sessions, key = { it.sessionId }) { session ->
                    SessionCard(session)
                }
            }
        }
    }
}

@Composable
private fun SessionCard(session: JellyfinSessionUi) {
    Panel(Modifier.fillMaxWidth()) {
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            session.seriesName?.let { series ->
                Text(
                    listOfNotNull(series, session.episodeLabel()).joinToString(" · "),
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }

            Text(
                session.itemName,
                style = MaterialTheme.typography.titleSmall,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )

            Text(
                "${session.user} · ${session.device} · ${session.client}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )

            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                val tone = if (session.paused) StateTone.LIGHT else StateTone.PRESENT
                OutlinedPill(
                    if (session.paused) "paused" else "playing",
                    color = tone.accent,
                    borderColor = tone.accent.copy(alpha = 0.4f),
                )
                OutlinedPill(session.itemType)
                session.playMethod?.let { OutlinedPill(it) }
            }
        }

        session.progress()?.let { progress ->
            Row(
                Modifier.fillMaxWidth().padding(top = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                LinearProgressIndicator(progress = { progress }, modifier = Modifier.weight(1f))

                Text(
                    "${duration(session.positionSeconds)} / ${duration(session.runtimeSeconds)}",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

private fun JellyfinSessionUi.episodeLabel(): String? {
    val season = season ?: return null
    val episode = episode ?: return null

    return "S%02dE%02d".format(season, episode)
}

private fun JellyfinSessionUi.progress(): Float? {
    val position = positionSeconds ?: return null
    val runtime = runtimeSeconds?.takeIf { it > 0 } ?: return null

    return (position / runtime).toFloat().coerceIn(0f, 1f)
}

private fun duration(seconds: Double?): String {
    val total = seconds?.toLong() ?: return "—"
    val hours = total / 3600
    val minutes = (total % 3600) / 60
    val secs = total % 60

    return if (hours > 0) "%d:%02d:%02d".format(hours, minutes, secs) else "%d:%02d".format(minutes, secs)
}
