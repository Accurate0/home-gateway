---@meta

---@class ZigbeeModel
---@field roles DeviceRole[]
---@field capabilities DeviceCapability[]?
---@field decode fun(payload: table<string, any>): DeviceReading
