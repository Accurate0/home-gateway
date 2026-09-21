---@meta

---@alias DeviceRole "battery"|"door"|"environment"|"light"|"smart_switch"|"presence"|"control_switch"
---@alias DeviceEnvironmentMetric "temperature"|"humidity"|"pressure"|"lux"|"uv_index"|"pm25"|"voc_index"
---@alias DeviceCapability DeviceEnvironmentMetric|"brightness"|"colour_temp"|"rgb"

---@class DoorFields
---@field contact boolean?

---@class LightFields
---@field state string?
---@field brightness integer?
---@field color_temp integer?
---@field color table?

---@class SmartSwitchFields
---@field state string?
---@field voltage integer?
---@field power integer?
---@field current number?
---@field energy number?

---@class PresenceFields
---@field presence boolean?
---@field sensor string?

---@class ControlSwitchFields
---@field action string?

---@class RobotVacuumFields
---@field status string?
---@field room string?
---@field battery integer?

---@class MediaPlayerFields
---@field state string?
---@field attributes table<string, any>?

---@class DeviceReading
---@field battery integer?
---@field door DoorFields?
---@field environment table<DeviceEnvironmentMetric, number>?
---@field plant table<string, number>?
---@field light LightFields?
---@field smart_switch SmartSwitchFields?
---@field presence PresenceFields?
---@field control_switch ControlSwitchFields?
---@field robot_vacuum RobotVacuumFields?
---@field media_player MediaPlayerFields?
---@field metrics table<string, number|string|boolean>?
