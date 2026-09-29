package net.infk8s.homegateway.dashboard

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase

@Database(
    entities = [SectionPosition::class, TilePosition::class],
    version = 1,
    exportSchema = false,
)
abstract class DashboardDatabase : RoomDatabase() {
    abstract fun dashboard(): DashboardDao

    companion object {
        fun build(context: Context): DashboardDatabase =
            Room.databaseBuilder(context, DashboardDatabase::class.java, "dashboard.db").build()
    }
}
