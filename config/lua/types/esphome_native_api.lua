---@meta

---@alias EsphomeNativeApiRole "battery"|"environment"|"plant"|"light"|"presence"|"media_player"
---@alias EsphomeNativeApiDomain "sensor"|"binary_sensor"|"text_sensor"|"light"|"media_player"

---@class MediaPlayerStateFields
---@field state string
---@field volume number
---@field muted boolean

---@class EsphomeNativeApiInput
---@field domain EsphomeNativeApiDomain
---@field object_id string
---@field payload number|boolean|string|LightFields|MediaPlayerStateFields
---@field entities table<string, number|boolean|string|LightFields|MediaPlayerStateFields>

---@class EsphomeNativeApiModel
---@field roles EsphomeNativeApiRole[]
---@field watchdog string?
---@field capabilities DeviceCapability[]?
---@field ranges DeviceRanges?
---@field decode fun(input: EsphomeNativeApiInput): DeviceReading
---@field encode DeviceEncoders?
