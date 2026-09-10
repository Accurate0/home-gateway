package net.infk8s.homegateway.notifications

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import java.time.OffsetDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import net.infk8s.homegateway.graphql.type.NotificationInteractionKind

@Composable
fun NotificationsScreen(
    state: NotificationsUiState,
    onRefresh: () -> Unit,
    onAcknowledge: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    when (state) {
        is NotificationsUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is NotificationsUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Text(state.message, color = MaterialTheme.colorScheme.error)
                AssistChip(onClick = onRefresh, label = { Text("Retry") })
            }
        }

        is NotificationsUiState.Loaded -> LazyColumn(
            modifier = modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            item(key = "header") {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text("Recent notifications", style = MaterialTheme.typography.titleMedium)
                    AssistChip(onClick = onRefresh, label = { Text("Refresh") })
                }
            }

            if (state.notifications.isEmpty()) {
                item(key = "empty") {
                    Text(
                        "No notifications yet",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            items(state.notifications, key = { it.id }) { notification ->
                NotificationCard(notification, onAcknowledge)
            }
        }
    }
}

@Composable
private fun NotificationCard(notification: NotificationUi, onAcknowledge: (String) -> Unit) {
    val awaitingAck = notification.acknowledgeable && notification.acknowledgedAt == null

    Card(Modifier.fillMaxWidth()) {
        Column(
            Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    notification.title,
                    style = MaterialTheme.typography.titleSmall,
                    modifier = Modifier.weight(1f),
                )
                Text(
                    formatTime(notification.createdAt),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            Text(notification.body, style = MaterialTheme.typography.bodyMedium)

            Text(
                notification.status(),
                style = MaterialTheme.typography.bodySmall,
                color = if (awaitingAck) {
                    MaterialTheme.colorScheme.error
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                },
            )

            if (awaitingAck) {
                AssistChip(
                    onClick = { onAcknowledge(notification.id) },
                    label = { Text("Acknowledge") },
                )
            }
        }
    }
}

private fun NotificationUi.status(): String = buildList {
    add(category.replaceFirstChar { it.uppercase() })

    if (sendCount > 1) add("sent ${sendCount}×")

    when {
        acknowledgedAt != null -> add("acknowledged ${formatTime(acknowledgedAt)}")
        acknowledgeable && nextReminderAt != null -> add("reminder ${formatTime(nextReminderAt)}")
        acknowledgeable -> add("not acknowledged")
    }

    interactions
        .filterNot { it == NotificationInteractionKind.ACKNOWLEDGED }
        .distinct()
        .forEach { add(it.rawValue.lowercase()) }
}.joinToString(" · ")

private val TIME_FORMAT: DateTimeFormatter = DateTimeFormatter.ofPattern("EEE d MMM, HH:mm")

private fun formatTime(value: String): String =
    runCatching {
        OffsetDateTime.parse(value).atZoneSameInstant(ZoneId.systemDefault()).format(TIME_FORMAT)
    }.getOrDefault(value)
