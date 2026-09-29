---@meta

---@alias TuyaRole "battery"|"garage_door"

---@class TuyaInput
---@field dps table<string, number|boolean|string> every data point the device has reported since connecting
---@field changed table<string, number|boolean|string> the data points in this report

---@class TuyaModel
---@field roles TuyaRole[]
---@field watchdog string?
---@field decode fun(input: TuyaInput): DeviceReading
---@field encode DeviceEncoders?
