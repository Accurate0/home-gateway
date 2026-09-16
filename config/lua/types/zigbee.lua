---@meta

---@alias ZigbeeRole "battery"|"door"|"environment"|"light"|"smart_switch"|"presence"|"control_switch"
---@alias ZigbeeEnvironmentMetric "temperature"|"humidity"|"pressure"|"lux"|"uv_index"|"pm25"|"voc_index"

---@class ZigbeeDoorReading
---@field contact boolean?

---@class ZigbeeLightReading
---@field state string?
---@field brightness integer?
---@field color_temp integer?
---@field color table?

---@class ZigbeeSmartSwitchReading
---@field state string?
---@field voltage integer?
---@field power integer?
---@field current number?
---@field energy number?

---@class ZigbeePresenceReading
---@field presence boolean?

---@class ZigbeeControlSwitchReading
---@field action string?

---@class ZigbeeReading
---@field battery integer?
---@field door ZigbeeDoorReading?
---@field environment table<ZigbeeEnvironmentMetric, number>?
---@field light ZigbeeLightReading?
---@field smart_switch ZigbeeSmartSwitchReading?
---@field presence ZigbeePresenceReading?
---@field control_switch ZigbeeControlSwitchReading?
---@field metrics table<string, number|string|boolean>?

---@class ZigbeeModel
---@field roles ZigbeeRole[]
---@field environment ZigbeeEnvironmentMetric[]?
---@field decode fun(payload: table<string, any>): ZigbeeReading
