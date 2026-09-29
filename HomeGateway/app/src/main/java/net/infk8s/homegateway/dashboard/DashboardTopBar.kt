package net.infk8s.homegateway.dashboard

import androidx.compose.foundation.layout.Box
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.painterResource
import net.infk8s.homegateway.R

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DashboardTopBar(
    mode: DashboardMode?,
    editing: Boolean,
    onToggleEditing: () -> Unit,
    onModeChange: (DashboardMode) -> Unit,
    onAddSection: () -> Unit,
    onReorderSections: () -> Unit,
    onSignOut: () -> Unit,
) {
    var menuOpen by remember { mutableStateOf(false) }

    TopAppBar(
        title = { Text(if (editing) "Rearrange" else "Home") },
        actions = {
            IconButton(onClick = onToggleEditing) {
                Icon(
                    painterResource(if (editing) R.drawable.ic_check else R.drawable.ic_edit),
                    contentDescription = if (editing) "Done" else "Rearrange",
                )
            }

            Box {
                IconButton(onClick = { menuOpen = true }) {
                    Icon(painterResource(R.drawable.ic_more), contentDescription = "More")
                }

                DropdownMenu(expanded = menuOpen, onDismissRequest = { menuOpen = false }) {
                    DashboardMode.entries.forEach { option ->
                        DropdownMenuItem(
                            text = { Text(option.label) },
                            leadingIcon = { RadioButton(selected = option == mode, onClick = null) },
                            onClick = {
                                menuOpen = false
                                onModeChange(option)
                            },
                        )
                    }

                    HorizontalDivider()

                    if (mode == DashboardMode.CUSTOM) {
                        DropdownMenuItem(
                            text = { Text("Add section") },
                            onClick = {
                                menuOpen = false
                                onAddSection()
                            },
                        )
                    }
                    DropdownMenuItem(
                        text = { Text("Reorder sections") },
                        onClick = {
                            menuOpen = false
                            onReorderSections()
                        },
                    )
                    DropdownMenuItem(
                        text = { Text("Sign out") },
                        onClick = {
                            menuOpen = false
                            onSignOut()
                        },
                    )
                }
            }
        },
    )
}
