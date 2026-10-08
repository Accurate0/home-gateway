---@meta

---@alias EsphomeNativeApiRole "battery"|"environment"|"plant"|"light"|"presence"|"media_player"|"air_purifier"
---@alias EsphomeNativeApiDomain "sensor"|"binary_sensor"|"text_sensor"|"light"|"media_player"|"fan"|"switch"|"number"|"select"

---@class MediaPlayerStateFields
---@field state string
---@field volume number
---@field muted boolean

---@class FanStateFields
---@field state "ON"|"OFF"
---@field speed integer
---@field preset string?

---@alias EsphomeNativeApiPayload number|boolean|string|LightFields|MediaPlayerStateFields|FanStateFields

---@class EsphomeNativeApiInput
---@field domain EsphomeNativeApiDomain
---@field object_id string
---@field payload EsphomeNativeApiPayload
---@field entities table<string, EsphomeNativeApiPayload>

---@class EsphomeNativeApiModel
---@field roles EsphomeNativeApiRole[]
---@field watchdog string?
---@field capabilities DeviceCapability[]?
---@field ranges DeviceRanges?
---@field decode fun(input: EsphomeNativeApiInput): DeviceReading
---@field encode DeviceEncoders?
