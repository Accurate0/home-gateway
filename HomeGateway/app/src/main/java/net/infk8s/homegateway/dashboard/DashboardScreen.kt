package net.infk8s.homegateway.dashboard

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import net.infk8s.homegateway.R
import net.infk8s.homegateway.graphql.EntitiesUiState
import net.infk8s.homegateway.graphql.EntitySectionUi
import net.infk8s.homegateway.graphql.EntityUi
import sh.calvin.reorderable.ReorderableItem
import sh.calvin.reorderable.rememberReorderableLazyGridState

@Composable
fun DashboardScreen(
    state: EntitiesUiState,
    controls: EntityControls,
    lightLevels: LightLevels,
    editing: Boolean,
    onMoveTile: (String, String) -> Unit,
    onRenameSection: (String, String) -> Unit,
    onDeleteSection: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    when (state) {
        is EntitiesUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is EntitiesUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is EntitiesUiState.Loaded -> DashboardGrid(
            state.mode,
            state.sections.filterNot {
                state.mode == DashboardMode.CUSTOM && !editing && it.key == DashboardLayout.UNSORTED_KEY
            },
            state.offline,
            controls,
            lightLevels,
            editing,
            onMoveTile,
            onRenameSection,
            onDeleteSection,
            modifier,
        )
    }
}

@Composable
private fun DashboardGrid(
    mode: DashboardMode,
    sections: List<EntitySectionUi>,
    offline: Set<String>,
    controls: EntityControls,
    lightLevels: LightLevels,
    editing: Boolean,
    onMoveTile: (String, String) -> Unit,
    onRenameSection: (String, String) -> Unit,
    onDeleteSection: (String) -> Unit,
    modifier: Modifier,
) {
    var selectedKey by rememberSaveable { mutableStateOf<String?>(null) }
    var editingSectionKey by rememberSaveable { mutableStateOf<String?>(null) }

    val haptics = LocalHapticFeedback.current
    val gridState = rememberLazyGridState()
    val reorderState = rememberReorderableLazyGridState(gridState) { from, to ->
        onMoveTile(from.key as String, to.key as String)
        haptics.performHapticFeedback(HapticFeedbackType.SegmentFrequentTick)
    }

    LazyVerticalGrid(
        columns = GridCells.Fixed(2),
        state = gridState,
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        sections.forEach { section ->
            val headerKey = DashboardLayout.headerKey(section.key)

            item(
                key = headerKey,
                span = { GridItemSpan(maxLineSpan) },
                contentType = "section",
            ) {
                ReorderableItem(reorderState, key = headerKey, enabled = editing && mode == DashboardMode.CUSTOM) {
                    SectionHeader(
                        section,
                        editable = editing && section.editable,
                        onEdit = { editingSectionKey = section.key },
                    )
                }
            }

            items(
                section.items,
                key = { it.key },
                span = {
                    GridItemSpan(if (it is EntityUi.Environment || it is EntityUi.Plant) maxLineSpan else 1)
                },
                contentType = { "tile" },
            ) { entity ->
                ReorderableItem(reorderState, key = entity.key, enabled = editing) { dragging ->
                    val elevation by animateDpAsState(if (dragging) 8.dp else 0.dp, label = "tile elevation")

                    EntityTile(
                        entity,
                        offline = entity.id in offline,
                        editing,
                        onClick = { entity.mainAction(controls)?.invoke() ?: run { selectedKey = entity.key } },
                        onLongClick = { selectedKey = entity.key },
                        dragHandle = {
                            Icon(
                                painterResource(R.drawable.ic_drag_handle),
                                contentDescription = "Reorder",
                                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier = Modifier
                                    .size(24.dp)
                                    .draggableHandle(
                                        onDragStarted = {
                                            haptics.performHapticFeedback(HapticFeedbackType.GestureThresholdActivate)
                                        },
                                        onDragStopped = {
                                            haptics.performHapticFeedback(HapticFeedbackType.GestureEnd)
                                        },
                                    ),
                            )
                        },
                        modifier = Modifier.shadow(elevation, RoundedCornerShape(16.dp)),
                    )
                }
            }
        }
    }

    val selected = selectedKey?.let { key ->
        sections.firstNotNullOfOrNull { section -> section.items.firstOrNull { it.key == key } }
    }

    if (selected != null && !editing) {
        EntityOptionsSheet(
            selected,
            offline = selected.id in offline,
            controls,
            lightLevels,
            onDismiss = { selectedKey = null },
        )
    }

    val editingSection = editingSectionKey?.let { key -> sections.firstOrNull { it.key == key && it.editable } }
    if (editingSection != null) {
        SectionNameDialog(
            title = "Edit section",
            initial = editingSection.title,
            confirmLabel = "Save",
            onConfirm = { name ->
                onRenameSection(editingSection.key, name)
                editingSectionKey = null
            },
            onDismiss = { editingSectionKey = null },
            onDelete = {
                onDeleteSection(editingSection.key)
                editingSectionKey = null
            },
        )
    }
}

@Composable
private fun SectionHeader(section: EntitySectionUi, editable: Boolean, onEdit: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().padding(top = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(section.title, style = MaterialTheme.typography.titleMedium)

        if (editable) {
            IconButton(onClick = onEdit) {
                Icon(
                    painterResource(R.drawable.ic_edit),
                    contentDescription = "Edit ${section.title}",
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(20.dp),
                )
            }
        }
    }
}
