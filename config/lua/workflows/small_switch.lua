local lights = gw.lib("lights")

local small_switch = {}

local FLOOR_LAMP = "floor-lamp-living-room"
local LAMPS_ON = "living-room-lamps-on"
local LAMPS_OFF = "living-room-lamps-off"
local AUTOMATIONS_TAG = "living-room"
local INDICATOR = "livingroom-motion"
local FLASH_SECONDS = 0.25
local FLASH_BRIGHTNESS = 128

local TAPS = {
	single = 1,
	double = 2,
	triple = 3,
	quadruple = 4,
}

local function toggle_lamps()
	local on = workflow.step("check", function()
		return lights.any_on({ FLOOR_LAMP })
	end, FLOOR_LAMP)

	local target = on and LAMPS_OFF or LAMPS_ON

	workflow.step("toggle", function()
		workflow.run(target)
	end, target)
end

local function toggle_automations()
	workflow.step("automations", function()
		workflow.set_enabled(AUTOMATIONS_TAG, "TOGGLE")
	end, AUTOMATIONS_TAG)
end

local handlers = {
	single = toggle_lamps,
	double = toggle_automations,
}

local function flash(taps)
	workflow.step("flash", function()
		local on = light.is_on(INDICATOR)
		local away = on and "OFF" or "ON"
		local back = on and "ON" or "OFF"

		if not on then
			light.set(INDICATOR, { state = "SET_BRIGHTNESS", value = FLASH_BRIGHTNESS })
		end

		for _ = 1, taps do
			light.set(INDICATOR, { state = away })
			gw.sleep(FLASH_SECONDS)
			light.set(INDICATOR, { state = back })
			gw.sleep(FLASH_SECONDS)
		end
	end, INDICATOR .. " x" .. taps)
end

function small_switch.tap()
	local taps = TAPS[event.action]

	if taps == nil then
		gw.log("no small switch handling for `" .. tostring(event.action) .. "`")
		return
	end

	local handler = handlers[event.action]

	if handler ~= nil then
		handler()
	end

	flash(taps)
end

return small_switch
