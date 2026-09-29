package net.infk8s.homegateway.dashboard

import androidx.annotation.DrawableRes
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.infk8s.homegateway.R
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.ui.theme.StateTone

@Composable
fun EntityTile(
    entity: EntityUi,
    editing: Boolean,
    onClick: () -> Unit,
    onLongClick: () -> Unit,
    dragHandle: @Composable () -> Unit,
    modifier: Modifier = Modifier,
) {
    val tone = entity.activeTone()
    val accent = tone?.accent
    val base = MaterialTheme.colorScheme.surfaceContainerLow

    Surface(
        shape = TileShape,
        color = accent?.copy(alpha = 0.15f)?.compositeOver(base) ?: base,
        border = BorderStroke(1.dp, accent?.copy(alpha = 0.5f) ?: MaterialTheme.colorScheme.outlineVariant),
        modifier = modifier
            .clip(TileShape)
            .combinedClickable(enabled = !editing, onClick = onClick, onLongClick = onLongClick),
    ) {
        if (entity is EntityUi.Environment) {
            EnvironmentContent(entity, editing, dragHandle)
        } else {
            StandardContent(entity, tone, editing, dragHandle)
        }
    }
}

@Composable
private fun StandardContent(
    entity: EntityUi,
    tone: StateTone?,
    editing: Boolean,
    dragHandle: @Composable () -> Unit,
) {
    Column(
        Modifier.padding(16.dp).heightIn(min = 124.dp),
        verticalArrangement = Arrangement.SpaceBetween,
    ) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.Top,
        ) {
            TileIcon(entity.icon(), tone)

            if (editing) {
                dragHandle()
            } else {
                entity.battery()?.let { BatteryLabel(it) }
            }
        }

        Spacer(Modifier.height(12.dp))

        Column {
            Text(
                entity.name,
                style = MaterialTheme.typography.titleSmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                entity.subtitle(),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )

            entity.pill()?.let {
                Spacer(Modifier.height(8.dp))
                StatePill(it)
            }
        }
    }
}

@Composable
private fun EnvironmentContent(
    entity: EntityUi.Environment,
    editing: Boolean,
    dragHandle: @Composable () -> Unit,
) {
    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.Top,
        ) {
            Row(
                Modifier.weight(1f),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                TileIcon(entity.icon(), null, size = 36.dp)

                Column {
                    Text(
                        entity.name,
                        style = MaterialTheme.typography.titleSmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        entity.subtitle(),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }

            if (editing) {
                dragHandle()
            } else {
                Text(
                    entity.temperature?.let { "%.1f°".format(it) } ?: "—",
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.SemiBold,
                )
            }
        }

        Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Text(
                    "Humidity",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    entity.humidity?.let { "%.0f%%".format(it) } ?: "—",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            LinearProgressIndicator(
                progress = { ((entity.humidity ?: 0.0) / 100.0).toFloat().coerceIn(0f, 1f) },
                modifier = Modifier.fillMaxWidth().height(6.dp),
                color = StateTone.PRESENT.accent.copy(alpha = 0.7f),
                trackColor = MaterialTheme.colorScheme.surfaceVariant,
                strokeCap = StrokeCap.Round,
                gapSize = 0.dp,
                drawStopIndicator = {},
            )
        }
    }
}

@Composable
private fun TileIcon(@DrawableRes icon: Int, tone: StateTone?, size: Dp = 40.dp) {
    Box(
        Modifier
            .size(size)
            .clip(RoundedCornerShape(12.dp))
            .background(tone?.accent ?: MaterialTheme.colorScheme.surfaceVariant),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            painterResource(icon),
            contentDescription = null,
            tint = tone?.onAccent ?: MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(20.dp),
        )
    }
}

@Composable
private fun BatteryLabel(percentage: Double) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            painterResource(R.drawable.ic_battery),
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(16.dp),
        )
        Text(
            "%.0f%%".format(percentage),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun StatePill(pill: TilePill) {
    val colors = MaterialTheme.colorScheme

    Surface(
        shape = CircleShape,
        color = when (pill.active) {
            true -> colors.onSurface
            false -> colors.surfaceVariant
            null -> Color.Transparent
        },
        contentColor = if (pill.active == true) colors.surface else colors.onSurfaceVariant,
        border = if (pill.active == null) BorderStroke(1.dp, colors.outlineVariant) else null,
    ) {
        Text(
            pill.label.uppercase(),
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.SemiBold,
            letterSpacing = 0.6.sp,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(horizontal = 10.dp, vertical = 2.dp),
        )
    }
}

private val TileShape = RoundedCornerShape(16.dp)
