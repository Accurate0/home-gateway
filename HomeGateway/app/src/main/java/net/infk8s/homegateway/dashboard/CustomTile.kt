package net.infk8s.homegateway.dashboard

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "custom_tile",
    foreignKeys = [
        ForeignKey(
            entity = CustomSection::class,
            parentColumns = ["id"],
            childColumns = ["section_id"],
            onDelete = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index("section_id")],
)
data class CustomTile(
    @PrimaryKey val key: String,
    @ColumnInfo(name = "section_id") val sectionId: String,
    val position: Int,
)
