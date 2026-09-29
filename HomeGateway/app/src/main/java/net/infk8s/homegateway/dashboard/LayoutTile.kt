package net.infk8s.homegateway.dashboard

import androidx.room.Entity

@Entity(tableName = "layout_tile", primaryKeys = ["mode", "key"])
data class LayoutTile(
    val mode: DashboardMode,
    val key: String,
    val position: Int,
)
