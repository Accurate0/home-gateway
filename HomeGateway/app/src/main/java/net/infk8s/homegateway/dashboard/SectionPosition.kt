package net.infk8s.homegateway.dashboard

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "section_position")
data class SectionPosition(
    @PrimaryKey val category: String,
    val position: Int,
)
