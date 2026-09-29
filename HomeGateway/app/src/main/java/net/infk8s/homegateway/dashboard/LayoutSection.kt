package net.infk8s.homegateway.dashboard

import androidx.room.Entity

@Entity(tableName = "layout_section", primaryKeys = ["mode", "key"])
data class LayoutSection(
    val mode: DashboardMode,
    val key: String,
    val position: Int,
)
