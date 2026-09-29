package net.infk8s.homegateway.dashboard

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "custom_section")
data class CustomSection(
    @PrimaryKey val id: String,
    val title: String,
    val position: Int,
)
