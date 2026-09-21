---@meta

---@alias EsphomeRole "environment"|"plant"|"light"|"presence"
---@alias EsphomeDomain "sensor"|"binary_sensor"|"light"

---@class EsphomeEntities
---@field sensor string[]?
---@field binary_sensor string[]?
---@field light string[]?

---@class EsphomeEntity
---@field domain EsphomeDomain
---@field object_id string
---@field state number|boolean|LightFields

---@class EsphomeModel
---@field roles EsphomeRole[]
---@field capabilities DeviceCapability[]?
---@field plant string[]?
---@field entities EsphomeEntities
---@field decode fun(entity: EsphomeEntity): DeviceReading
