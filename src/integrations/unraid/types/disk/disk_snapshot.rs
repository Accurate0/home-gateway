use super::{DiskKind, DiskStatus};

#[derive(Debug, Clone, PartialEq)]
pub struct DiskSnapshot {
    pub name: String,
    pub kind: DiskKind,
    pub status: Option<DiskStatus>,
    pub temperature: Option<i32>,
    pub size_kb: Option<i64>,
    pub fs_size_kb: Option<i64>,
    pub fs_free_kb: Option<i64>,
    pub fs_used_kb: Option<i64>,
    pub errors: Option<i64>,
    pub spinning: Option<bool>,
}

impl DiskSnapshot {
    pub fn usage_percent(&self) -> Option<f64> {
        let size = self.fs_size_kb.filter(|size| *size > 0)?;
        let used = self.fs_used_kb?;

        Some(used as f64 / size as f64 * 100.0)
    }
}
