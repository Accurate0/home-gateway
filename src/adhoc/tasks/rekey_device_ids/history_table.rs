use sqlx::{Postgres, Transaction};
use strum::{Display, EnumIter};

use super::rekey::Rekey;
use super::renames;

#[derive(Debug, Clone, Copy, Display, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum HistoryTable {
    DeviceBattery,
    DeviceMetric,
    RobotVacuumEvents,
    TemperatureSensor,
    PlantSensor,
    DerivedDoorEvents,
}

impl HistoryTable {
    pub fn renames(self) -> &'static [Rekey] {
        match self {
            HistoryTable::DeviceBattery
            | HistoryTable::DeviceMetric
            | HistoryTable::RobotVacuumEvents => renames::DEVICES,
            HistoryTable::TemperatureSensor => renames::ENVIRONMENTS,
            HistoryTable::PlantSensor => renames::PLANTS,
            HistoryTable::DerivedDoorEvents => renames::DOORS,
        }
    }

    pub async fn rekey_batch(
        self,
        tx: &mut Transaction<'static, Postgres>,
        rekey: Rekey,
        batch_size: i64,
    ) -> Result<u64, sqlx::Error> {
        sqlx::query!("SET LOCAL timescaledb.max_tuples_decompressed_per_dml_transaction = 0")
            .execute(&mut **tx)
            .await?;

        let Rekey { from, to } = rekey;

        let result = match self {
            HistoryTable::DeviceBattery => {
                sqlx::query!(
                    "UPDATE device_battery SET device_id = $2 \
                     WHERE device_id = $1 AND time IN ( \
                         SELECT time FROM device_battery WHERE device_id = $1 \
                         ORDER BY time LIMIT $3 \
                     )",
                    from,
                    to,
                    batch_size
                )
                .execute(&mut **tx)
                .await?
            }
            HistoryTable::DeviceMetric => {
                sqlx::query!(
                    "UPDATE device_metric SET device_id = $2 \
                     WHERE device_id = $1 AND time IN ( \
                         SELECT time FROM device_metric WHERE device_id = $1 \
                         ORDER BY time LIMIT $3 \
                     )",
                    from,
                    to,
                    batch_size
                )
                .execute(&mut **tx)
                .await?
            }
            HistoryTable::RobotVacuumEvents => {
                sqlx::query!(
                    "UPDATE robot_vacuum_events SET device_id = $2 \
                     WHERE device_id = $1 AND time IN ( \
                         SELECT time FROM robot_vacuum_events WHERE device_id = $1 \
                         ORDER BY time LIMIT $3 \
                     )",
                    from,
                    to,
                    batch_size
                )
                .execute(&mut **tx)
                .await?
            }
            HistoryTable::TemperatureSensor => {
                sqlx::query!(
                    "UPDATE temperature_sensor SET id = $2 \
                     WHERE id = $1 AND time IN ( \
                         SELECT time FROM temperature_sensor WHERE id = $1 \
                         ORDER BY time LIMIT $3 \
                     )",
                    from,
                    to,
                    batch_size
                )
                .execute(&mut **tx)
                .await?
            }
            HistoryTable::PlantSensor => {
                sqlx::query!(
                    "UPDATE plant_sensor SET id = $2 \
                     WHERE id = $1 AND time IN ( \
                         SELECT time FROM plant_sensor WHERE id = $1 \
                         ORDER BY time LIMIT $3 \
                     )",
                    from,
                    to,
                    batch_size
                )
                .execute(&mut **tx)
                .await?
            }
            HistoryTable::DerivedDoorEvents => {
                sqlx::query!(
                    "UPDATE derived_door_events SET id = $2 \
                     WHERE id = $1 AND time IN ( \
                         SELECT time FROM derived_door_events WHERE id = $1 \
                         ORDER BY time LIMIT $3 \
                     )",
                    from,
                    to,
                    batch_size
                )
                .execute(&mut **tx)
                .await?
            }
        };

        Ok(result.rows_affected())
    }
}
