---@meta

---@alias gw.EnableState "ENABLED"|"DISABLED"|"TOGGLE"

---@alias gw.EnvMetric "temperature"|"humidity"|"pressure"|"lux"|"uv_index"

---@alias gw.LightState { state: "ON" }|{ state: "OFF" }|{ state: "TOGGLE" }|{ value: integer, state: "SET_BRIGHTNESS" }|{ value: integer, on_off?: boolean, state: "INCREASE_BRIGHTNESS" }|{ value: integer, on_off?: boolean, state: "DECREASE_BRIGHTNESS" }|{ value: integer, state: "INCREASE_COLOUR_TEMPERATURE" }|{ value: integer, state: "DECREASE_COLOUR_TEMPERATURE" }|{ state: "STOP_COLOUR_TEMPERATURE" }|{ state: "STOP_BRIGHTNESS" }

---@alias gw.Mode "home"|"away"|"vacation"|"guest"

---@alias gw.NotifyAction { label: string, action: gw.NotifyActionKind }

---@alias gw.NotifyActionKind { workflow: string, type: "run_workflow" }|{ seconds: integer, type: "snooze" }|{ type: "dismiss" }|{ type: "acknowledge" }

---@alias gw.NotifyCategory "alarm"|"door"|"watchdog"|"general"

---@alias gw.SunPeriod "day"|"night"

---@alias gw.SwitchState "ON"|"OFF"|"TOGGLE"

---@alias gw.VacuumCommand "start"|"stop"|"dock"

---@class gw.EnergyInterval
---@field used number
---@field exported number
---@field at integer

---@class gw.EnvironmentReading
---@field temperature? number
---@field humidity? number
---@field pressure? number
---@field lux? number
---@field uv_index? number

---@class gw.Forecast
---@field days gw.ForecastDay[]
---@field hours gw.ForecastHour[]

---@class gw.ForecastDay
---@field date_time string
---@field code string
---@field description string
---@field emoji string
---@field min integer
---@field max integer
---@field uv? number
---@field rain_probability? integer
---@field rain_start_range? integer
---@field rain_end_range? integer
---@field rain_range_code? string
---@field wind_max_speed? number
---@field first_light? string
---@field sunrise? string
---@field sunset? string
---@field last_light? string

---@class gw.ForecastHour
---@field date_time string
---@field temperature? number
---@field wind_speed? number
---@field wind_direction? number
---@field wind_direction_text? string

---@class gw.FuelSite
---@field site_id integer
---@field name string
---@field brand string
---@field suburb string
---@field postcode integer
---@field address string
---@field price number
---@field price_tomorrow? number
---@field latitude number
---@field longitude number

---@class gw.HttpRequest
---@field url string
---@field method? string
---@field headers? table<string, string>
---@field body? string

---@class gw.HttpResponse
---@field status integer
---@field ok boolean
---@field body string

---@class gw.JellyfinSession
---@field user string
---@field device string
---@field client string
---@field item string
---@field item_type string
---@field series? string
---@field season? integer
---@field episode? integer
---@field position? number
---@field runtime? number
---@field paused boolean

---@class gw.LowBattery
---@field device string
---@field level number

---@class gw.MediaPlayerState
---@field state string
---@field app? string
---@field source? string
---@field title? string
---@field series? string
---@field content_type? string
---@field volume? number
---@field muted? boolean
---@field updated_at integer

---@class gw.MetricSample
---@field value any
---@field at integer

---@class gw.Notification
---@field message string
---@field title? string
---@field category gw.NotifyCategory
---@field tag? string
---@field actions? gw.NotifyAction[]
---@field acknowledge? gw.NotifyAcknowledge

---@class gw.NotifyAcknowledge
---@field remind_after string
---@field reminders integer

---@class gw.Request
---@field method string
---@field path string
---@field params table<string, string>
---@field query table<string, any>
---@field headers table<string, string>
---@field body? string
---@field json? any

---@class gw.Response
---@field status? integer
---@field headers? table<string, string>
---@field body any

---@class gw.SolarAverages
---@field last_15_mins number
---@field last_1_hour number
---@field last_3_hours number

---@class gw.TimeNow
---@field iso string
---@field epoch integer
---@field date string
---@field hour integer
---@field minute integer
---@field weekday string

---@class gw.TransperthDeparture
---@field line string
---@field headsign string
---@field platform? string
---@field minutes integer
---@field delay? integer
---@field live boolean

---@class gw.UnifiClient
---@field name string
---@field connected boolean
---@field since integer

---@class gw.VacuumState
---@field state? string
---@field battery? integer
---@field fan_speed? string
---@field room? string
---@field updated_at integer

---@class gw.WoolworthsTrackedPrice
---@field product_id integer
---@field price? number

---@type table<string, any>
event = {}

---@type gw.Request
request = {}

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

---@param query string
---@param variables? table<string, any>
---@return any
function gw.graphql(query, variables) end

---@param name string
---@return any
function gw.lib(name) end

---Requires the `workflow:write` scope.
---@param key string
---@param seconds integer
---@return boolean
function gw.cooldown(key, seconds) end

---Requires the `workflow:run` scope.
---@param name string
---@param payload? table<string, any>
function gw.emit(name, payload) end

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

---Requires the `workflow:write` scope.
---@param tag string
---@param state gw.EnableState
function workflow.set_enabled(tag, state) end

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

---@class gw.api.holidays
holidays = {}

---Requires the `holiday:read` scope.
---@param date? string
---@return string?
function holidays.on(date) end

---Requires the `holiday:read` scope.
---@param date? string
---@return boolean
function holidays.in_week(date) end

---@class gw.api.state
state = {}

---Requires the `workflow:read` scope.
---@param key string
---@return any
function state.get(key) end

---Requires the `workflow:write` scope.
---@param key string
---@param value any
function state.set(key, value) end

---Requires the `workflow:write` scope.
---@param key string
function state.clear(key) end

---Requires the `workflow:write` scope.
---@param key string
---@param by? integer
---@return integer
function state.incr(key, by) end

---@class gw.api.time
time = {}

---@return gw.TimeNow
function time.now() end

---@param start string
---@param finish string
---@return boolean
function time.between(start, finish) end

---@return boolean
function time.weekend() end

---@param iso string
---@return integer
function time.parse(iso) end

---@param epoch integer
---@param pattern string
---@return string
function time.format(epoch, pattern) end

---@class gw.api.device
device = {}

---Requires the `device:read` scope.
---@param device string
---@return integer?
function device.last_seen(device) end

---Requires the `device:read` scope.
---@param minutes integer
---@return string[]
function device.offline(minutes) end

---Requires the `device:read` scope.
---@param device string
---@param key string
---@return gw.MetricSample?
function device.metric(device, key) end

---Requires the `device:read` scope.
---@param device string
---@param key string
---@param epoch integer
---@return gw.MetricSample[]
function device.metric_since(device, key, epoch) end

---@class gw.api.battery
battery = {}

---Requires the `device:read` scope.
---@param device string
---@return number?
function battery.level(device) end

---Requires the `device:read` scope.
---@param threshold number
---@return gw.LowBattery[]
function battery.low(threshold) end

---@class gw.api.vacuum
vacuum = {}

---Requires the `robot_vacuum:read` scope.
---@param device string
---@return gw.VacuumState?
function vacuum.state(device) end

---Requires the `robot_vacuum:write` scope.
---@param device string
---@param command gw.VacuumCommand
function vacuum.command(device, command) end

---@class gw.api.energy
energy = {}

---Requires the `energy:read` scope.
---@param epoch integer
---@return gw.EnergyInterval[]
function energy.since(epoch) end

---@class gw.api.woolworths
woolworths = {}

---Requires the `woolworths:read` scope.
---@param product_id integer
---@return number?
function woolworths.price(product_id) end

---Requires the `woolworths:read` scope.
---@return gw.WoolworthsTrackedPrice[]
function woolworths.tracked() end

---@class gw.api.transperth
transperth = {}

---Requires the `transperth:read` scope.
---@param route string
---@return gw.TransperthDeparture[]?
function transperth.next(route) end

---@class gw.api.re
re = {}

---@param pattern string
---@param text string
---@return boolean
function re.test(pattern, text) end

---@param pattern string
---@param text string
---@return table?
function re.match(pattern, text) end

---@param pattern string
---@param text string
---@return string[]
function re.find_all(pattern, text) end

---@param pattern string
---@param text string
---@param replacement string
---@return string
function re.replace(pattern, text, replacement) end

---@param pattern string
---@param text string
---@return string[]
function re.split(pattern, text) end

---@class gw.api.alarm
alarm = {}

---Requires the `alarm:read` scope.
---@return integer?
function alarm.next() end

---@class gw.api.vacation
vacation = {}

---Requires the `vacation:read` scope.
---@return boolean
function vacation.active() end

---Requires the `vacation:write` scope.
---@param armed boolean
function vacation.arm(armed) end

---@class gw.api.unifi
unifi = {}

---Requires the `unifi:read` scope.
---@param client string
---@return boolean
function unifi.home(client) end

---Requires the `unifi:read` scope.
---@return gw.UnifiClient[]
function unifi.clients() end

---@class gw.api.media
media = {}

---Requires the `jellyfin:read` scope.
---@return gw.JellyfinSession[]
function media.jellyfin() end

---Requires the `media.player:read` scope.
---@param device string
---@return gw.MediaPlayerState?
function media.player(device) end

---@class gw.api.flag
flag = {}

---Requires the `feature_flag:read` scope.
---@param name string
---@param default boolean
---@return boolean
function flag.enabled(name, default) end

---Requires the `feature_flag:read` scope.
---@param name string
---@return table<string, any>?
function flag.get(name) end

---@class gw.api.s3
s3 = {}

---Requires the `s3:read` scope.
---@param key string
---@return string?
function s3.get(key) end

---Requires the `s3:write` scope.
---@param key string
---@param body string
---@param content_type? string
function s3.put(key, body, content_type) end

---Requires the `s3:read` scope.
---@param prefix string
---@return string[]
function s3.list(prefix) end

---@class gw.api.weather
weather = {}

---Requires the `weather:read` scope.
---@param location? string
---@return gw.Forecast?
function weather.forecast(location) end

---Requires the `weather:read` scope.
---@param location? string
---@return gw.ForecastDay?
function weather.today(location) end

---@class gw.api.fuel
fuel = {}

---Requires the `fuelwatch:read` scope.
---@param postcode? integer
---@param limit? integer
---@return gw.FuelSite[]
function fuel.cheapest(postcode, limit) end

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
