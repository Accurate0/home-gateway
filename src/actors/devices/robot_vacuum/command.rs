use crate::decoding::DeviceRoleName;
use crate::device_command::{self, CommandTargets, DeviceCommandError, Outbound};
use crate::settings::RobotVacuumSettings;
use crate::settings::workflow::VacuumCommand;

pub async fn send(
    targets: &CommandTargets<'_>,
    settings: &RobotVacuumSettings,
    command: VacuumCommand,
) -> Result<(), DeviceCommandError> {
    let instruction = settings.commands.get(command).to_owned();

    device_command::send(
        targets,
        &settings.address,
        DeviceRoleName::RobotVacuum,
        Outbound::Text(instruction),
    )
    .await
}
