package net.infk8s.homegateway.picker

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import net.infk8s.homegateway.dashboard.activeTone
import net.infk8s.homegateway.dashboard.details
import net.infk8s.homegateway.dashboard.icon
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.graphql.EntitySnapshot
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.ui.TitleTopBar

private sealed interface PickerState {
    data object Loading : PickerState
    data object SignedOut : PickerState
    data class Error(val message: String) : PickerState
    data class Loaded(val snapshot: EntitySnapshot) : PickerState
}

@Composable
fun EntityPickerScreen(
    title: String,
    onPick: (EntityUi) -> Unit,
    onCancel: () -> Unit,
) {
    val context = LocalContext.current
    val state by produceState<PickerState>(PickerState.Loading) {
        value = if (!context.gateway.auth.signedIn.value) {
            PickerState.SignedOut
        } else {
            try {
                PickerState.Loaded(context.gateway.entities.fetch())
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                PickerState.Error(e.message ?: "Failed to load devices")
            }
        }
    }

    Scaffold(topBar = { TitleTopBar(title, onBack = onCancel) }) { padding ->
        val modifier = Modifier.padding(padding)

        when (val current = state) {
            PickerState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) { CircularProgressIndicator() }
            PickerState.SignedOut -> Message("Open Home Gateway and sign in first.", modifier)
            is PickerState.Error -> Message(current.message, modifier)
            is PickerState.Loaded -> EntityList(current.snapshot, onPick, modifier)
        }
    }
}

@Composable
private fun EntityList(snapshot: EntitySnapshot, onPick: (EntityUi) -> Unit, modifier: Modifier) {
    var query by rememberSaveable { mutableStateOf("") }

    val matching = snapshot.entities.filter { entity ->
        query.isBlank() || listOfNotNull(entity.name, entity.room, entity.id).any { it.contains(query, ignoreCase = true) }
    }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        item(key = "search") {
            OutlinedTextField(
                value = query,
                onValueChange = { query = it },
                label = { Text("Search") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth().padding(bottom = 8.dp),
            )
        }

        snapshot.categories.forEach { category ->
            val items = matching.filter { it.category == category.category }.sortedBy { it.name.lowercase() }
            if (items.isEmpty()) {
                return@forEach
            }

            item(key = "category:${category.category.rawValue}") {
                Text(
                    category.title.uppercase(),
                    style = MaterialTheme.typography.labelSmall,
                    fontWeight = FontWeight.SemiBold,
                    letterSpacing = 1.5.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 16.dp, bottom = 4.dp),
                )
            }

            items(items, key = { it.key }) { entity -> EntityRow(entity, onClick = { onPick(entity) }) }
        }
    }
}

@Composable
private fun EntityRow(entity: EntityUi, onClick: () -> Unit) {
    val tone = entity.activeTone()

    Row(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .clickable(onClick = onClick)
            .padding(horizontal = 8.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            Modifier
                .size(36.dp)
                .clip(RoundedCornerShape(10.dp))
                .background(tone?.accent ?: MaterialTheme.colorScheme.surfaceVariant),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                painterResource(entity.icon()),
                contentDescription = null,
                tint = tone?.onAccent ?: MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.size(18.dp),
            )
        }

        Column(Modifier.weight(1f)) {
            Text(entity.name, style = MaterialTheme.typography.bodyLarge, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(
                listOfNotNull(entity.room, entity.details()).joinToString(" · "),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun Message(text: String, modifier: Modifier) {
    Box(modifier.fillMaxSize().padding(24.dp), Alignment.Center) {
        Text(text, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}
