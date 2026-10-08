---@meta

---@alias DeviceRole "battery"|"door"|"environment"|"light"|"smart_switch"|"presence"|"control_switch"
---@alias DeviceEnvironmentMetric "temperature"|"humidity"|"pressure"|"lux"|"uv_index"|"pm25"|"voc_index"
---@alias DeviceCapability DeviceEnvironmentMetric|"brightness"|"colour_temp"|"rgb"

---@class DeviceRange
---@field min integer
---@field max integer

---@class DeviceRanges
---@field brightness DeviceRange? native brightness scale; required with the `brightness` capability
---@field colour_temp DeviceRange? mireds the product accepts; required with the `colour_temp` capability

---@class DoorFields
---@field contact boolean?

---@class LightFields
---@field state string?
---@field brightness integer?
---@field color_temp integer?
---@field color table?

---@class LightSetCommand
---@field type "set"
---@field on boolean?
---@field brightness integer?
---@field colour_temp integer?
---@field colour string?

---@class LightToggleCommand
---@field type "toggle"

---@class LightBrightnessMoveCommand
---@field type "brightness_move"
---@field value integer
---@field on_off boolean

---@class LightColourTempMoveCommand
---@field type "colour_temp_move"
---@field value integer

---@alias LightCommand LightSetCommand|LightToggleCommand|LightBrightnessMoveCommand|LightColourTempMoveCommand

---@class LightCurrent
---@field on boolean
---@field brightness integer?
---@field colour_temp integer?
---@field colour string?

---@class LightEncodeInput
---@field command LightCommand
---@field current LightCurrent

---@alias LightEncoder fun(input: LightEncodeInput): table?

---@alias GarageDoorState "open"|"opening"|"closed"|"closing"

---@class GarageDoorEncodeInput
---@field command "open"|"close"
---@field current GarageDoorState?

---@alias GarageDoorEncoder fun(input: GarageDoorEncodeInput): table?

---@alias AirPurifierMode "manual"|"sleep"|"auto"

---@class AirPurifierCommand
---@field command "turn_on"|"turn_off"|"set_mode"|"set_speed"|"set_display"
---@field mode AirPurifierMode?
---@field speed integer?
---@field on boolean?

---@class AirPurifierCurrent
---@field on boolean?
---@field mode AirPurifierMode?
---@field speed integer?
---@field display boolean?

---@class AirPurifierEncodeInput
---@field command AirPurifierCommand
---@field current AirPurifierCurrent

---@alias AirPurifierEncoder fun(input: AirPurifierEncodeInput): table?

---@class DeviceEncoders
---@field light LightEncoder?
---@field smart_switch LightEncoder?
---@field garage_door GarageDoorEncoder?
---@field air_purifier AirPurifierEncoder?

---@class SmartSwitchFields
---@field state string?
---@field voltage integer?
---@field power integer?
---@field current number?
---@field energy number?

---@class PresenceFields
---@field presence boolean?
---@field sensor string?

---@class PlantFields
---@field soil_moisture number?

---@class ControlSwitchFields
---@field action string?

---@class RobotVacuumFields
---@field status string?
---@field room string?
---@field battery integer?
---@field fan_speed string?
---@field clean_area number?
---@field clean_count integer?
---@field attributes table<string, any>?

---@class MediaPlayerFields
---@field state string?
---@field attributes table<string, any>?

---@class GarageDoorFields
---@field state GarageDoorState?
---@field contact boolean?

---@class AirPurifierFields
---@field state string?
---@field mode AirPurifierMode?
---@field speed integer?
---@field pm25 number?
---@field filter_life number?
---@field display boolean?

---@class DeviceReading
---@field battery integer?
---@field door DoorFields?
---@field environment table<DeviceEnvironmentMetric, number>?
---@field plant PlantFields?
---@field light LightFields?
---@field smart_switch SmartSwitchFields?
---@field presence PresenceFields?
---@field control_switch ControlSwitchFields?
---@field robot_vacuum RobotVacuumFields?
---@field media_player MediaPlayerFields?
---@field garage_door GarageDoorFields?
---@field air_purifier AirPurifierFields?
---@field metrics table<string, number|string|boolean>?
