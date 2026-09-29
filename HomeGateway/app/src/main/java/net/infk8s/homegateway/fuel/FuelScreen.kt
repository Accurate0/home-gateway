package net.infk8s.homegateway.fuel

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.infk8s.homegateway.ui.OutlinedPill
import net.infk8s.homegateway.ui.Panel
import net.infk8s.homegateway.ui.theme.StateTone

@Composable
fun FuelScreen(state: FuelUiState, modifier: Modifier = Modifier) {
    when (state) {
        is FuelUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is FuelUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is FuelUiState.Loaded -> if (state.sites.isEmpty()) {
            Box(modifier.fillMaxSize(), Alignment.Center) {
                Text(
                    "No fuel prices stored yet",
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
                itemsIndexed(state.sites, key = { _, site -> site.siteId }) { index, site ->
                    SiteCard(site, cheapest = index == 0)
                }
            }
        }
    }
}

@Composable
private fun SiteCard(site: FuelSiteUi, cheapest: Boolean) {
    val context = LocalContext.current
    val accent = StateTone.PRESENT.accent

    Panel(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .clickable {
                val position = "${site.latitude},${site.longitude}"
                val uri = Uri.parse("geo:$position?q=$position(${Uri.encode(site.name)})")

                context.startActivity(Intent(Intent.ACTION_VIEW, uri))
            },
        borderColor = if (cheapest) accent.copy(alpha = 0.5f) else MaterialTheme.colorScheme.outlineVariant,
    ) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(site.name, style = MaterialTheme.typography.titleSmall, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(
                    "${site.address}, ${site.suburb}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )

                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    OutlinedPill(site.brand)

                    if (cheapest) {
                        OutlinedPill("cheapest", color = accent, borderColor = accent.copy(alpha = 0.4f))
                    }
                }
            }

            Column(horizontalAlignment = Alignment.End) {
                Text(
                    "%.1f¢".format(site.price),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = if (cheapest) accent else MaterialTheme.colorScheme.onSurface,
                )

                site.priceTomorrow?.let { tomorrow ->
                    Text(
                        "tomorrow %.1f¢ %s".format(tomorrow, trend(site.price, tomorrow)),
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

private fun trend(today: Double, tomorrow: Double): String = when {
    tomorrow > today -> "↑"
    tomorrow < today -> "↓"
    else -> "="
}
