package net.infk8s.homegateway.dashboard

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "tile_position")
data class TilePosition(
    @PrimaryKey val key: String,
    val position: Int,
)
