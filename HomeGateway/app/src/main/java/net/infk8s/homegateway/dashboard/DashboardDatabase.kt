package net.infk8s.homegateway.dashboard

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase
import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

@Database(
    entities = [LayoutSection::class, LayoutTile::class, CustomSection::class, CustomTile::class],
    version = 2,
    exportSchema = false,
)
abstract class DashboardDatabase : RoomDatabase() {
    abstract fun dashboard(): DashboardDao

    companion object {
        fun build(context: Context): DashboardDatabase =
            Room.databaseBuilder(context, DashboardDatabase::class.java, "dashboard.db")
                .addMigrations(MIGRATION_1_2)
                .build()

        private val MIGRATION_1_2 = object : Migration(1, 2) {
            override fun migrate(db: SupportSQLiteDatabase) {
                db.execSQL(
                    "CREATE TABLE IF NOT EXISTS `layout_section` (`mode` TEXT NOT NULL, `key` TEXT NOT NULL, " +
                        "`position` INTEGER NOT NULL, PRIMARY KEY(`mode`, `key`))",
                )
                db.execSQL(
                    "INSERT INTO `layout_section` (`mode`, `key`, `position`) " +
                        "SELECT 'TYPE', `category`, `position` FROM `section_position`",
                )
                db.execSQL("DROP TABLE `section_position`")

                db.execSQL(
                    "CREATE TABLE IF NOT EXISTS `layout_tile` (`mode` TEXT NOT NULL, `key` TEXT NOT NULL, " +
                        "`position` INTEGER NOT NULL, PRIMARY KEY(`mode`, `key`))",
                )
                db.execSQL(
                    "INSERT INTO `layout_tile` (`mode`, `key`, `position`) " +
                        "SELECT 'TYPE', `key`, `position` FROM `tile_position`",
                )
                db.execSQL("DROP TABLE `tile_position`")

                db.execSQL(
                    "CREATE TABLE IF NOT EXISTS `custom_section` (`id` TEXT NOT NULL, `title` TEXT NOT NULL, " +
                        "`position` INTEGER NOT NULL, PRIMARY KEY(`id`))",
                )
                db.execSQL(
                    "CREATE TABLE IF NOT EXISTS `custom_tile` (`key` TEXT NOT NULL, `section_id` TEXT NOT NULL, " +
                        "`position` INTEGER NOT NULL, PRIMARY KEY(`key`), FOREIGN KEY(`section_id`) " +
                        "REFERENCES `custom_section`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE )",
                )
                db.execSQL(
                    "CREATE INDEX IF NOT EXISTS `index_custom_tile_section_id` ON `custom_tile` (`section_id`)",
                )
            }
        }
    }
}
