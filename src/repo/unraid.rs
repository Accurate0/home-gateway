use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

use crate::integrations::unraid::types::{ArraySnapshot, DiskSnapshot, UnraidSnapshot};

#[derive(Clone)]
pub struct UnraidRepo {
    db: Pool<Postgres>,
}

pub struct StoredUnraidSnapshot {
    pub recorded_at: DateTime<Utc>,
    pub snapshot: UnraidSnapshot,
}

pub struct UnraidArrayHistoryRow {
    pub time: DateTime<Utc>,
    pub kilobytes_used: i64,
    pub kilobytes_total: i64,
    pub parity_status: String,
    pub parity_progress: Option<i64>,
}

pub struct UnraidDiskHistoryRow {
    pub time: DateTime<Utc>,
    pub status: Option<String>,
    pub temperature: Option<i32>,
    pub fs_used_kb: Option<i64>,
    pub fs_size_kb: Option<i64>,
    pub errors: Option<i64>,
    pub spinning: Option<bool>,
}

struct ArrayRow {
    time: DateTime<Utc>,
    state: String,
    kilobytes_free: i64,
    kilobytes_used: i64,
    kilobytes_total: i64,
    parity_status: String,
    parity_progress: Option<i64>,
    parity_errors: Option<i64>,
    parity_running: bool,
}

struct DiskRow {
    disk: String,
    kind: String,
    status: Option<String>,
    temperature: Option<i32>,
    size_kb: Option<i64>,
    fs_size_kb: Option<i64>,
    fs_free_kb: Option<i64>,
    fs_used_kb: Option<i64>,
    errors: Option<i64>,
    spinning: Option<bool>,
}

struct DiskColumns {
    disk: Vec<String>,
    kind: Vec<String>,
    status: Vec<Option<String>>,
    temperature: Vec<Option<i32>>,
    size_kb: Vec<Option<i64>>,
    fs_size_kb: Vec<Option<i64>>,
    fs_free_kb: Vec<Option<i64>>,
    fs_used_kb: Vec<Option<i64>>,
    errors: Vec<Option<i64>>,
    spinning: Vec<Option<bool>>,
}

impl DiskColumns {
    fn new(disks: &[DiskSnapshot]) -> Self {
        Self {
            disk: disks.iter().map(|disk| disk.name.clone()).collect(),
            kind: disks.iter().map(|disk| disk.kind.to_string()).collect(),
            status: disks
                .iter()
                .map(|disk| disk.status.map(|status| status.to_string()))
                .collect(),
            temperature: disks.iter().map(|disk| disk.temperature).collect(),
            size_kb: disks.iter().map(|disk| disk.size_kb).collect(),
            fs_size_kb: disks.iter().map(|disk| disk.fs_size_kb).collect(),
            fs_free_kb: disks.iter().map(|disk| disk.fs_free_kb).collect(),
            fs_used_kb: disks.iter().map(|disk| disk.fs_used_kb).collect(),
            errors: disks.iter().map(|disk| disk.errors).collect(),
            spinning: disks.iter().map(|disk| disk.spinning).collect(),
        }
    }
}

fn decode<T: FromStr>(column: &str, value: &str) -> Result<T, sqlx::Error> {
    value.parse().map_err(|_| sqlx::Error::ColumnDecode {
        index: column.to_owned(),
        source: format!("unknown value `{value}`").into(),
    })
}

impl DiskRow {
    fn into_snapshot(self) -> Result<DiskSnapshot, sqlx::Error> {
        Ok(DiskSnapshot {
            kind: decode("kind", &self.kind)?,
            status: self
                .status
                .as_deref()
                .map(|status| decode("status", status))
                .transpose()?,
            name: self.disk,
            temperature: self.temperature,
            size_kb: self.size_kb,
            fs_size_kb: self.fs_size_kb,
            fs_free_kb: self.fs_free_kb,
            fs_used_kb: self.fs_used_kb,
            errors: self.errors,
            spinning: self.spinning,
        })
    }
}

impl UnraidRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.unraid.record", fields(disks = snapshot.disks.len()), err)]
    pub async fn record(&self, snapshot: &UnraidSnapshot) -> Result<(), sqlx::Error> {
        let mut tx = self.db.begin().await?;
        let time = Utc::now();
        let array = &snapshot.array;

        sqlx::query!(
            r#"
            INSERT INTO unraid_array_events (
                state, kilobytes_free, kilobytes_used, kilobytes_total,
                parity_status, parity_progress, parity_errors, parity_running, "time"
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            array.state.to_string(),
            array.kilobytes_free,
            array.kilobytes_used,
            array.kilobytes_total,
            array.parity_status.to_string(),
            array.parity_progress,
            array.parity_errors,
            array.parity_running,
            time,
        )
        .execute(&mut *tx)
        .await?;

        let columns = DiskColumns::new(&snapshot.disks);

        sqlx::query!(
            r#"
            INSERT INTO unraid_disk_events (
                disk, kind, status, temperature, size_kb,
                fs_size_kb, fs_free_kb, fs_used_kb, errors, spinning, "time"
            )
            SELECT *, $11::TIMESTAMPTZ FROM UNNEST(
                $1::TEXT[], $2::TEXT[], $3::TEXT[], $4::INTEGER[], $5::BIGINT[],
                $6::BIGINT[], $7::BIGINT[], $8::BIGINT[], $9::BIGINT[], $10::BOOLEAN[]
            )
            "#,
            &columns.disk,
            &columns.kind,
            &columns.status as &[Option<String>],
            &columns.temperature as &[Option<i32>],
            &columns.size_kb as &[Option<i64>],
            &columns.fs_size_kb as &[Option<i64>],
            &columns.fs_free_kb as &[Option<i64>],
            &columns.fs_used_kb as &[Option<i64>],
            &columns.errors as &[Option<i64>],
            &columns.spinning as &[Option<bool>],
            time,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    #[tracing::instrument(skip_all, name = "db.unraid.latest", err)]
    pub async fn latest(&self) -> Result<Option<StoredUnraidSnapshot>, sqlx::Error> {
        let array = sqlx::query_as!(
            ArrayRow,
            r#"
            SELECT "time", state, kilobytes_free, kilobytes_used, kilobytes_total,
                   parity_status, parity_progress, parity_errors, parity_running
            FROM unraid_array_events
            ORDER BY "time" DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.db)
        .await?;

        let Some(array) = array else {
            return Ok(None);
        };

        let disks = sqlx::query_as!(
            DiskRow,
            r#"
            SELECT disk, kind, status, temperature, size_kb,
                   fs_size_kb, fs_free_kb, fs_used_kb, errors, spinning
            FROM unraid_disk_events
            WHERE "time" = $1
            ORDER BY disk
            "#,
            array.time,
        )
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(DiskRow::into_snapshot)
        .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(StoredUnraidSnapshot {
            recorded_at: array.time,
            snapshot: UnraidSnapshot {
                array: ArraySnapshot {
                    state: decode("state", &array.state)?,
                    kilobytes_free: array.kilobytes_free,
                    kilobytes_used: array.kilobytes_used,
                    kilobytes_total: array.kilobytes_total,
                    parity_status: decode("parity_status", &array.parity_status)?,
                    parity_progress: array.parity_progress,
                    parity_errors: array.parity_errors,
                    parity_running: array.parity_running,
                },
                disks,
            },
        }))
    }

    #[tracing::instrument(skip_all, name = "db.unraid.array_history", err)]
    pub async fn array_history(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<UnraidArrayHistoryRow>, sqlx::Error> {
        sqlx::query_as!(
            UnraidArrayHistoryRow,
            r#"
            SELECT "time", kilobytes_used, kilobytes_total, parity_status, parity_progress
            FROM unraid_array_events
            WHERE "time" >= $1
            ORDER BY "time"
            "#,
            since,
        )
        .fetch_all(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.unraid.disk_history", err)]
    pub async fn disk_history(
        &self,
        disk: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<UnraidDiskHistoryRow>, sqlx::Error> {
        sqlx::query_as!(
            UnraidDiskHistoryRow,
            r#"
            SELECT "time", status, temperature, fs_used_kb, fs_size_kb, errors, spinning
            FROM unraid_disk_events
            WHERE disk = $1 AND "time" >= $2
            ORDER BY "time"
            "#,
            disk,
            since,
        )
        .fetch_all(&self.db)
        .await
    }
}
