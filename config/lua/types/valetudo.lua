---@meta

---@alias ValetudoRole "robot_vacuum"|"battery"

---@class ValetudoInput: MqttInput
---@field vars { address: string }
---@field payload table<string, any>|string

---@class ValetudoModel
---@field protocol "valetudo"
---@field roles ValetudoRole[]
---@field commands { robot_vacuum: RobotVacuumCommands }
---@field decode fun(input: ValetudoInput): DeviceReading
