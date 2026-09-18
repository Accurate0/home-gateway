---@meta

---@class ZigbeeModel
---@field roles DeviceRole[]
---@field environment DeviceEnvironmentMetric[]?
---@field decode fun(payload: table<string, any>): DeviceReading
