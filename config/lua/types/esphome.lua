---@meta

---@alias EsphomeRole "environment"|"plant"|"light"|"presence"
---@alias EsphomeDomain "sensor"|"binary_sensor"|"light"

---@class EsphomeEntities
---@field sensor string[]?
---@field binary_sensor string[]?
---@field light string[]?

---@class EsphomeInput: MqttInput
---@field vars { address: string, domain: EsphomeDomain, object_id: string }
---@field payload number|boolean|LightFields

---@class EsphomeModel
---@field protocol "esphome"
---@field roles EsphomeRole[]
---@field watchdog string?
---@field capabilities DeviceCapability[]?
---@field ranges DeviceRanges?
---@field plant string[]?
---@field entities EsphomeEntities
---@field decode fun(input: EsphomeInput): DeviceReading
---@field encode DeviceEncoders?
