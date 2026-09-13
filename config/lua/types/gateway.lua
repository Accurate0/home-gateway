---@meta

---@alias gw.EnvMetric "temperature"|"humidity"|"pressure"|"lux"|"uv_index"

---@alias gw.LightState { state: "ON" }|{ state: "OFF" }|{ state: "TOGGLE" }|{ value: integer, state: "SET_BRIGHTNESS" }|{ value: integer, on_off?: boolean, state: "INCREASE_BRIGHTNESS" }|{ value: integer, on_off?: boolean, state: "DECREASE_BRIGHTNESS" }|{ value: integer, state: "INCREASE_COLOUR_TEMPERATURE" }|{ value: integer, state: "DECREASE_COLOUR_TEMPERATURE" }|{ state: "STOP_COLOUR_TEMPERATURE" }|{ state: "STOP_BRIGHTNESS" }

---@alias gw.Mode "home"|"away"|"vacation"|"guest"

---@alias gw.NotifyCategory "alarm"|"door"|"watchdog"|"general"

---@alias gw.SunPeriod "day"|"night"

---@alias gw.SwitchState "ON"|"OFF"|"TOGGLE"

---@class gw.EnvironmentReading
---@field temperature? number
---@field humidity? number
---@field pressure? number
---@field lux? number
---@field uv_index? number

---@class gw.HttpRequest
---@field url string
---@field method? string
---@field headers? table<string, string>
---@field body? string

---@class gw.HttpResponse
---@field status integer
---@field ok boolean
---@field body string

---@class gw.Notification
---@field message string
---@field title? string
---@field category gw.NotifyCategory

---@class gw.SolarAverages
---@field last_15_mins number
---@field last_1_hour number
---@field last_3_hours number

---@type table<string, any>
event = {}

---@type table<string, any>
input = {}

---@type table<string, any>
lua = {}

---@class gw.api.gw
---@field event_id string
---@field origin string
---@field dry_run boolean
gw = {}

---@param message string
function gw.log(message) end

---@param seconds number
function gw.sleep(seconds) end

---@param scope string
---@return boolean
function gw.has(scope) end

---@param scope string
function gw.require(scope) end

---Requires the `http:write` scope.
---@param request gw.HttpRequest
---@return gw.HttpResponse
function gw.http(request) end

---@param name string
---@return any
function gw.lib(name) end

---@class gw.api.json
json = {}

---@param raw string
---@return any
function json.decode(raw) end

---@param value any
---@return string
function json.encode(value) end

---@class gw.api.light
light = {}

---Requires the `light:write` scope.
---@param device string
---@param command gw.LightState
function light.set(device, command) end

---Requires the `light:read` scope.
---@param device string
---@return boolean
function light.is_on(device) end

---@class gw.api.switch
switch = {}

---Requires the `switch:write` scope.
---@param device string
---@param command gw.SwitchState
function switch.set(device, command) end

---Requires the `switch:read` scope.
---@param device string
---@return boolean
function switch.is_on(device) end

---@class gw.api.presence
presence = {}

---Requires the `presence:read` scope.
---@param device string
---@return boolean
function presence.at(device) end

---@class gw.api.door
door = {}

---Requires the `door:read` scope.
---@param device string
---@return boolean
function door.is_open(device) end

---@class gw.api.environment
environment = {}

---Requires the `environment:read` scope.
---@param device string
---@param metric gw.EnvMetric
---@return number?
function environment.get(device, metric) end

---Requires the `environment:read` scope.
---@param device string
---@return gw.EnvironmentReading?
function environment.reading(device) end

---@class gw.api.notify
notify = {}

---Requires the `push:write` scope.
---@param notification gw.Notification
function notify.send(notification) end

---@class gw.api.mqtt
mqtt = {}

---Requires the `mqtt:write` scope.
---@param topic string
---@param payload string
---@param retain? boolean
function mqtt.publish(topic, payload, retain) end

---@class gw.api.workflow
workflow = {}

---Requires the `workflow:run` scope.
---@param name string
---@param with? table<string, any>
function workflow.run(name, with) end

---Requires the `workflow:read` scope.
---@return gw.Mode
function workflow.mode() end

---Requires the `workflow:write` scope.
---@param mode gw.Mode
function workflow.set_mode(mode) end

---@class gw.api.sun
sun = {}

---@return gw.SunPeriod
function sun.period() end

---@param period gw.SunPeriod
---@return boolean
function sun.is(period) end

---@class gw.api.solar
solar = {}

---Requires the `solar:read` scope.
---@return number?
function solar.current() end

---Requires the `solar:read` scope.
---@return gw.SolarAverages
function solar.averages() end

---@class gw.api.home_assistant
home_assistant = {}

---Requires the `home_assistant:write` scope.
---@param service string
---@param data? table
function home_assistant.call(service, data) end

---Requires the `home_assistant:read` scope.
---@param entity_id string
---@return any
function home_assistant.state(entity_id) end
