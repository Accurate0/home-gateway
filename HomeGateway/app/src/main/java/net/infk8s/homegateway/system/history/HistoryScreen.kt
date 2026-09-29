package net.infk8s.homegateway.system.history

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.infk8s.homegateway.ui.Panel
import net.infk8s.homegateway.ui.theme.StateTone

@Composable
fun HistoryScreen(
    state: HistoryUiState,
    range: HistoryRange,
    healthy: ClosedFloatingPointRange<Double>,
    onRange: (HistoryRange) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier.fillMaxSize()) {
        Row(
            Modifier
                .fillMaxWidth()
                .horizontalScroll(rememberScrollState())
                .padding(horizontal = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            HistoryRange.entries.forEach { option ->
                FilterChip(
                    selected = option == range,
                    onClick = { onRange(option) },
                    label = { Text(option.label) },
                )
            }
        }

        when (state) {
            is HistoryUiState.Loading -> Box(Modifier.fillMaxSize(), Alignment.Center) {
                CircularProgressIndicator()
            }

            is HistoryUiState.Error -> Box(Modifier.fillMaxSize(), Alignment.Center) {
                Text(state.message, color = MaterialTheme.colorScheme.error)
            }

            is HistoryUiState.Loaded -> if (state.series.isEmpty()) {
                Box(Modifier.fillMaxSize(), Alignment.Center) {
                    Text(
                        "No history in this range",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(16.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    items(state.series, key = { it.id }) { series ->
                        SeriesCard(series, range, healthy)
                    }
                }
            }
        }
    }
}

@Composable
private fun SeriesCard(series: HistorySeries, range: HistoryRange, healthy: ClosedFloatingPointRange<Double>) {
    val tone = series.current?.let { if (it in healthy) StateTone.PRESENT else StateTone.OPEN }
    val colour = tone?.accent ?: MaterialTheme.colorScheme.onSurfaceVariant

    Panel(Modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text(
                    series.name,
                    style = MaterialTheme.typography.titleSmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    series.room ?: series.id,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }

            Text(
                series.current?.let { "%.0f%%".format(it) } ?: "—",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = colour,
            )
        }

        HistoryChart(
            points = series.points,
            since = range.since(),
            valueRange = PERCENT_RANGE,
            guides = listOf(healthy.start, healthy.endInclusive)
                .filter { it > PERCENT_RANGE.start && it < PERCENT_RANGE.endInclusive },
            lineColour = colour,
            guideColour = MaterialTheme.colorScheme.outlineVariant,
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 12.dp)
                .height(72.dp),
        )
    }
}

private val PERCENT_RANGE = 0.0..100.0
