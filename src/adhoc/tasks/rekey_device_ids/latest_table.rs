use sqlx::{Postgres, Transaction};
use strum::{Display, EnumIter};

use super::rekey::Rekey;
use super::renames;

#[derive(Debug, Clone, Copy, Display, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum LatestTable {
    DeviceBatteryLatest,
    EinkDisplay,
    LatestRobotVacuumState,
    MediaPlayerState,
    LatestDeviceMetric,
    DeviceLastSeen,
    LatestTemperatureSensor,
    LatestPlantSensor,
}

impl LatestTable {
    pub fn renames(self) -> &'static [Rekey] {
        match self {
            LatestTable::DeviceBatteryLatest
            | LatestTable::EinkDisplay
            | LatestTable::LatestRobotVacuumState
            | LatestTable::MediaPlayerState
            | LatestTable::LatestDeviceMetric
            | LatestTable::DeviceLastSeen => renames::DEVICES,
            LatestTable::LatestTemperatureSensor => renames::ENVIRONMENTS,
            LatestTable::LatestPlantSensor => renames::PLANTS,
        }
    }

    pub async fn rekey(
        self,
        tx: &mut Transaction<'static, Postgres>,
        rekey: Rekey,
    ) -> Result<u64, sqlx::Error> {
        let Rekey { from, to } = rekey;

        let (superseded, moved) = match self {
            LatestTable::DeviceBatteryLatest => {
                let superseded = sqlx::query!(
                    "DELETE FROM device_battery_latest WHERE device_id = $1 \
                     AND EXISTS (SELECT 1 FROM device_battery_latest WHERE device_id = $2)",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                let moved = sqlx::query!(
                    "UPDATE device_battery_latest SET device_id = $2 WHERE device_id = $1",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                (superseded, moved)
            }
            LatestTable::EinkDisplay => {
                let superseded = sqlx::query!(
                    "DELETE FROM eink_display WHERE device_id = $1 \
                     AND EXISTS (SELECT 1 FROM eink_display WHERE device_id = $2)",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                let moved = sqlx::query!(
                    "UPDATE eink_display SET device_id = $2 WHERE device_id = $1",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                (superseded, moved)
            }
            LatestTable::LatestRobotVacuumState => {
                let superseded = sqlx::query!(
                    "DELETE FROM latest_robot_vacuum_state WHERE device_id = $1 \
                     AND EXISTS (SELECT 1 FROM latest_robot_vacuum_state WHERE device_id = $2)",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                let moved = sqlx::query!(
                    "UPDATE latest_robot_vacuum_state SET device_id = $2 WHERE device_id = $1",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                (superseded, moved)
            }
            LatestTable::MediaPlayerState => {
                let superseded = sqlx::query!(
                    "DELETE FROM media_player_state WHERE device_id = $1 \
                     AND EXISTS (SELECT 1 FROM media_player_state WHERE device_id = $2)",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                let moved = sqlx::query!(
                    "UPDATE media_player_state SET device_id = $2 WHERE device_id = $1",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                (superseded, moved)
            }
            LatestTable::LatestDeviceMetric => {
                let moved = sqlx::query!(
                    "UPDATE latest_device_metric SET device_id = $2 WHERE device_id = $1",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                return Ok(moved.rows_affected());
            }
            LatestTable::DeviceLastSeen => {
                let superseded = sqlx::query!(
                    "DELETE FROM device_last_seen WHERE split_part(device_key, ':', 2) = $1 \
                     AND EXISTS ( \
                         SELECT 1 FROM device_last_seen current \
                         WHERE current.device_key = split_part(device_last_seen.device_key, ':', 1) || ':' || $2 \
                     )",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                let moved = sqlx::query!(
                    "UPDATE device_last_seen SET device_key = split_part(device_key, ':', 1) || ':' || $2 \
                     WHERE split_part(device_key, ':', 2) = $1",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                (superseded, moved)
            }
            LatestTable::LatestTemperatureSensor => {
                let superseded = sqlx::query!(
                    "DELETE FROM latest_temperature_sensor WHERE entity_id = $1 \
                     AND EXISTS (SELECT 1 FROM latest_temperature_sensor WHERE entity_id = $2)",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                let moved = sqlx::query!(
                    "UPDATE latest_temperature_sensor SET entity_id = $2 WHERE entity_id = $1",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                (superseded, moved)
            }
            LatestTable::LatestPlantSensor => {
                let superseded = sqlx::query!(
                    "DELETE FROM latest_plant_sensor WHERE entity_id = $1 \
                     AND EXISTS (SELECT 1 FROM latest_plant_sensor WHERE entity_id = $2)",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                let moved = sqlx::query!(
                    "UPDATE latest_plant_sensor SET entity_id = $2 WHERE entity_id = $1",
                    from,
                    to
                )
                .execute(&mut **tx)
                .await?;

                (superseded, moved)
            }
        };

        Ok(superseded.rows_affected() + moved.rows_affected())
    }
}
