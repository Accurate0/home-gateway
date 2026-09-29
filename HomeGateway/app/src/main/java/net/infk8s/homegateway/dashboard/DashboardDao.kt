package net.infk8s.homegateway.dashboard

import androidx.room.Dao
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Upsert

@Dao
interface DashboardDao {
    @Query("SELECT * FROM layout_section")
    suspend fun sections(): List<LayoutSection>

    @Query("SELECT * FROM layout_tile")
    suspend fun tiles(): List<LayoutTile>

    @Query("SELECT * FROM custom_section")
    suspend fun customSections(): List<CustomSection>

    @Query("SELECT * FROM custom_tile")
    suspend fun customTiles(): List<CustomTile>

    @Upsert
    suspend fun upsertSections(rows: List<LayoutSection>)

    @Upsert
    suspend fun upsertTiles(rows: List<LayoutTile>)

    @Upsert
    suspend fun upsertCustomSections(rows: List<CustomSection>)

    @Upsert
    suspend fun upsertCustomTiles(rows: List<CustomTile>)

    @Query("DELETE FROM custom_tile WHERE `key` IN (:keys)")
    suspend fun deleteCustomTiles(keys: List<String>)

    @Query("DELETE FROM custom_section WHERE id = :id")
    suspend fun deleteCustomSection(id: String)

    @Transaction
    suspend fun placeCustomTiles(placed: List<CustomTile>, unplaced: List<String>, unsorted: List<LayoutTile>) {
        deleteCustomTiles(unplaced)
        upsertCustomTiles(placed)
        upsertTiles(unsorted)
    }

    @Transaction
    suspend fun seedCustom(sections: List<CustomSection>, tiles: List<CustomTile>) {
        upsertCustomSections(sections)
        upsertCustomTiles(tiles)
    }
}
