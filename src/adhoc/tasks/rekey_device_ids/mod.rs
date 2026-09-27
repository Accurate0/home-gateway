mod history_table;
mod latest_table;
mod rekey;
mod renames;

use strum::IntoEnumIterator;

use crate::adhoc::{AdhocTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;

use history_table::HistoryTable;
use latest_table::LatestTable;

pub struct RekeyDeviceIds;

#[async_trait::async_trait]
impl AdhocTask for RekeyDeviceIds {
    fn ordinal(&self) -> i64 {
        3
    }

    fn name(&self) -> &'static str {
        "rekey_device_ids"
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<(), AdhocTaskError> {
        let batch_size = ctx.settings.adhoc.batch_size;

        Ok(rekey_all(ctx, batch_size).await?)
    }
}

pub async fn rekey_all(ctx: &mut AdhocTaskContext<'_>, batch_size: i64) -> Result<(), sqlx::Error> {
    for table in LatestTable::iter() {
        let mut total = 0;

        for &rekey in table.renames() {
            total += table.rekey(ctx.tx, rekey).await?;
        }

        tracing::info!("rekeyed {total} rows in {table}");

        ctx.commit_batch().await?;
    }

    for table in HistoryTable::iter() {
        for &rekey in table.renames() {
            let mut total = 0;

            loop {
                let rows = table.rekey_batch(ctx.tx, rekey, batch_size).await?;

                ctx.commit_batch().await?;

                total += rows;

                if rows < batch_size as u64 {
                    break;
                }

                tracing::info!("rekeyed {rows} rows in {table} for {rekey} ({total} so far)");
            }

            tracing::info!("rekeyed {total} rows in {table} for {rekey}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::rekey::Rekey;
    use super::renames::{DEVICES, DOORS, ENVIRONMENTS, PLANTS};
    use crate::device_registry::{RawDevice, is_device_id};

    const MAPS: [(&str, &[Rekey]); 4] = [
        ("devices", DEVICES),
        ("environments", ENVIRONMENTS),
        ("plants", PLANTS),
        ("doors", DOORS),
    ];

    #[test]
    fn no_old_id_is_renamed_twice() {
        for (name, map) in MAPS {
            let mut seen = HashSet::new();

            for rekey in map {
                assert!(
                    seen.insert(rekey.from),
                    "{name}: `{}` is renamed twice",
                    rekey.from
                );
            }
        }
    }

    #[test]
    fn every_new_id_follows_the_scheme() {
        for (name, map) in MAPS {
            for rekey in map {
                assert!(
                    is_device_id(rekey.to),
                    "{name}: `{}` breaks the scheme",
                    rekey.to
                );
            }
        }
    }

    #[test]
    fn every_new_id_is_a_configured_device() {
        let mut configured = HashSet::new();

        for entry in std::fs::read_dir("config/devices").unwrap() {
            let path = entry.unwrap().path();

            if path.file_name().is_some_and(|name| name == "index.yaml") {
                continue;
            }

            let devices: Vec<RawDevice> =
                serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

            configured.extend(devices.into_iter().map(|device| device.id));
        }

        for (name, map) in MAPS {
            for rekey in map {
                assert!(
                    configured.contains(rekey.to),
                    "{name}: `{}` is not a configured device",
                    rekey.to
                );
            }
        }
    }
}
