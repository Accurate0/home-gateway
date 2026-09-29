package net.infk8s.homegateway.dashboard

import androidx.room.Dao
import androidx.room.Query
import androidx.room.Upsert

@Dao
interface DashboardDao {
    @Query("SELECT * FROM section_position")
    suspend fun sections(): List<SectionPosition>

    @Query("SELECT * FROM tile_position")
    suspend fun tiles(): List<TilePosition>

    @Upsert
    suspend fun upsertSections(rows: List<SectionPosition>)

    @Upsert
    suspend fun upsertTiles(rows: List<TilePosition>)
}
