---@meta

---@alias ZigbeeRole "battery"|"door"|"environment"|"light"|"smart_switch"|"presence"|"control_switch"

---@class ZigbeeInput: MqttInput
---@field vars { name: string }
---@field payload table<string, any>

---@class ZigbeeModel
---@field protocol "zigbee"
---@field roles ZigbeeRole[]
---@field capabilities DeviceCapability[]?
---@field decode fun(input: ZigbeeInput): DeviceReading
