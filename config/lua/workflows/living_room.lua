local lights = gw.lib("lights")

local living_room = {}

local SENSOR = "livingroom-motion"
local FLOOR_LAMP = "floor-lamp-living-room"
local TABLE_LAMP = "living-room-table-lamp"
local LAMPS = { FLOOR_LAMP, TABLE_LAMP }

function living_room.occupied(settings)
	if sun.is("night", settings.sunset_offset) then
		workflow.step("lamps_on", function()
			lights.all(LAMPS, "ON")
		end)
	end
end

function living_room.vacated()
	workflow.step("lamps_off", function()
		lights.all(LAMPS, "OFF")
	end)
end

function living_room.sunset(settings)
	if not presence.seen_within(SENSOR, settings.vacancy) then
		gw.log("nobody in the living room at sunset")
		return
	end

	workflow.step("fade_up", function()
		if light.is_on(FLOOR_LAMP) then
			gw.log(FLOOR_LAMP .. " is already on")
			return
		end

		light.set(FLOOR_LAMP, { state = "SET_BRIGHTNESS", value = 1 })
		light.set(FLOOR_LAMP, { state = "INCREASE_BRIGHTNESS", value = settings.fade_rate })
	end, FLOOR_LAMP)

	workflow.step("table_lamp", function()
		if light.is_on(TABLE_LAMP) then
			gw.log(TABLE_LAMP .. " is already on")
			return
		end

		light.set(TABLE_LAMP, { state = "ON" })
	end, TABLE_LAMP)
end

return living_room
