---@meta

---@alias HomeAssistantRole "battery"|"door"|"environment"|"presence"|"robot_vacuum"|"media_player"

---@class HomeAssistantEntity
---@field entity_id string
---@field state string
---@field attributes table<string, any>

---@class HomeAssistantModel
---@field roles HomeAssistantRole[]
---@field capabilities DeviceCapability[]?
---@field entities string[]?
---@field decode fun(entity: HomeAssistantEntity): DeviceReading
