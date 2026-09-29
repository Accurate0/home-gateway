package net.infk8s.homegateway.transperth

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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.infk8s.homegateway.ui.OutlinedPill
import net.infk8s.homegateway.ui.Panel
import net.infk8s.homegateway.ui.clock
import net.infk8s.homegateway.ui.relative
import net.infk8s.homegateway.ui.theme.StateTone

@Composable
fun TransperthScreen(state: TransperthUiState, modifier: Modifier = Modifier) {
    when (state) {
        is TransperthUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is TransperthUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is TransperthUiState.Loaded -> if (state.routes.isEmpty()) {
            Box(modifier.fillMaxSize(), Alignment.Center) {
                Text(
                    "No routes configured",
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
                items(state.routes, key = { it.id }) { route ->
                    RouteCard(route)
                }
            }
        }
    }
}

@Composable
private fun RouteCard(route: RouteUi) {
    Panel(Modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text(
                    "${route.origin} → ${route.destination}",
                    style = MaterialTheme.typography.titleSmall,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                route.updatedAt?.let {
                    Text(
                        "updated ${it.relative()}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            if (route.stale) {
                val tone = StateTone.OPEN
                OutlinedPill("stale", color = tone.accent, borderColor = tone.accent.copy(alpha = 0.4f))
            }
        }

        if (route.departures.isEmpty()) {
            Text(
                "No upcoming departures",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(top = 12.dp),
            )
        }

        route.departures.forEachIndexed { index, departure ->
            if (index > 0) {
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            }

            DepartureRow(departure, Modifier.padding(top = if (index == 0) 12.dp else 8.dp, bottom = 8.dp))
        }
    }
}

@Composable
private fun DepartureRow(departure: DepartureUi, modifier: Modifier) {
    Row(
        modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(
                departure.headsign,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                listOfNotNull(
                    departure.line,
                    departure.platform?.let { "platform $it" },
                    departure.departsAt?.clock(),
                ).joinToString(" · "),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }

        Column(horizontalAlignment = Alignment.End) {
            Text(
                if (departure.minutesAway <= 0) "now" else "${departure.minutesAway} min",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = if (departure.live) StateTone.PRESENT.accent else MaterialTheme.colorScheme.onSurface,
            )

            when {
                !departure.live -> DelayText("scheduled", late = false)
                (departure.delayMinutes ?: 0) > 0 -> DelayText("${departure.delayMinutes} min late", late = true)
                else -> DelayText("on time", late = false)
            }
        }
    }
}

@Composable
private fun DelayText(label: String, late: Boolean) {
    Text(
        label,
        style = MaterialTheme.typography.labelSmall,
        color = if (late) StateTone.OPEN.accent else MaterialTheme.colorScheme.onSurfaceVariant,
    )
}
