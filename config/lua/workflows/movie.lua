local movie = {}

local FLOOR_LAMP = "floor-lamp-living-room"
local TABLE_LAMP = "living-room-table-lamp"
local OCCUPANCY_TAG = "living-room-occupancy"
local LIGHTS_OFF = "living-room-lamps-off"
local TABLE_LAMP_DELAY = 5

local function lights_off()
	workflow.step("occupancy", function()
		workflow.set_enabled(OCCUPANCY_TAG, "DISABLED")
	end, OCCUPANCY_TAG)

	workflow.step("lights_off", function()
		workflow.run(LIGHTS_OFF)
	end, LIGHTS_OFF)
end

local function paused(settings)
	workflow.step("pause_light", function()
		light.set(FLOOR_LAMP, { state = "SET_BRIGHTNESS", value = settings.pause_brightness })
	end, FLOOR_LAMP)
end

local function stopped(settings)
	workflow.step("occupancy", function()
		workflow.set_enabled(OCCUPANCY_TAG, "ENABLED")
	end, OCCUPANCY_TAG)

	workflow.step("fade_up", function()
		if not light.is_on(FLOOR_LAMP) then
			light.set(FLOOR_LAMP, { state = "SET_BRIGHTNESS", value = 1 })
		end

		light.set(FLOOR_LAMP, { state = "INCREASE_BRIGHTNESS", value = settings.fade_rate })
	end, FLOOR_LAMP)

	gw.defer(TABLE_LAMP_DELAY, TABLE_LAMP .. " on", function()
		if light.is_on(TABLE_LAMP) then
			gw.log(TABLE_LAMP .. " is already on")
			return
		end

		light.set(TABLE_LAMP, { state = "ON" })
	end)
end

local handlers = {
	started = lights_off,
	resumed = lights_off,
	paused = paused,
	stopped = stopped,
}

function movie.playback(settings)
	local handler = handlers[event.state]

	if handler == nil then
		gw.log("no movie handling for jellyfin `" .. tostring(event.state) .. "`")
		return
	end

	gw.log("movie " .. event.state .. ": " .. event.item)

	handler(settings)
end

return movie
