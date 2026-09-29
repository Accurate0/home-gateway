package net.infk8s.homegateway.woolworths

import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.infk8s.homegateway.system.history.HistoryChart
import net.infk8s.homegateway.system.history.HistoryPoint
import net.infk8s.homegateway.system.history.HistoryRange
import net.infk8s.homegateway.ui.Panel
import net.infk8s.homegateway.ui.theme.StateTone

@Composable
fun WoolworthsScreen(
    state: WoolworthsUiState,
    histories: Map<Int, List<HistoryPoint>>,
    range: HistoryRange,
    onExpand: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    when (state) {
        is WoolworthsUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is WoolworthsUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is WoolworthsUiState.Loaded -> if (state.products.isEmpty()) {
            Box(modifier.fillMaxSize(), Alignment.Center) {
                Text(
                    "No tracked products",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        } else {
            var expanded by rememberSaveable { mutableStateOf<Int?>(null) }

            LazyColumn(
                modifier = modifier.fillMaxSize(),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                items(state.products, key = { it.productId }) { product ->
                    ProductCard(
                        product,
                        expanded = expanded == product.productId,
                        history = histories[product.productId],
                        range = range,
                        onClick = {
                            expanded = if (expanded == product.productId) null else product.productId
                            if (expanded != null) {
                                onExpand(product.productId)
                            }
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun ProductCard(
    product: WoolworthsProductUi,
    expanded: Boolean,
    history: List<HistoryPoint>?,
    range: HistoryRange,
    onClick: () -> Unit,
) {
    Panel(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .clickable(onClick = onClick),
    ) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                product.name,
                style = MaterialTheme.typography.titleSmall,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )

            Text(
                "$%.2f".format(product.price),
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
            )
        }

        if (expanded) {
            PriceHistory(history, range)
        }
    }
}

@Composable
private fun PriceHistory(history: List<HistoryPoint>?, range: HistoryRange) {
    Column(Modifier.fillMaxWidth().padding(top = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        when {
            history == null -> LinearProgressIndicator(Modifier.fillMaxWidth())

            history.isEmpty() -> Text(
                "No price changes in the last ${range.label}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            else -> {
                val low = history.minOf { it.value }
                val high = history.maxOf { it.value }
                val padding = ((high - low) * CHART_PADDING).coerceAtLeast(MIN_PADDING)

                HistoryChart(
                    points = history,
                    since = range.since(),
                    valueRange = (low - padding)..(high + padding),
                    guides = emptyList(),
                    lineColour = StateTone.INFO.accent,
                    guideColour = MaterialTheme.colorScheme.outlineVariant,
                    modifier = Modifier.fillMaxWidth().height(72.dp),
                )

                Text(
                    "Last ${range.label} · low $%.2f · high $%.2f".format(low, high),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

private const val CHART_PADDING = 0.15
private const val MIN_PADDING = 0.1
