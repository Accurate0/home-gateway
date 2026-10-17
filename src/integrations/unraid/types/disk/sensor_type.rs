use crate::integrations::unraid::schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, cynic::Enum)]
pub enum SensorType {
    CpuPackage,
    CpuCore,
    Motherboard,
    Chipset,
    Gpu,
    Disk,
    Nvme,
    Ambient,
    Vrm,
    Custom,
}
