local living_room = test.load("workflows/living_room")

local FLOOR_LAMP = "floor-lamp-living-room"
local TABLE_LAMP = "living-room-table-lamp"
local PURIFIER = "living-room-purifier"

local AUTO = { PURIFIER, { command = "set_mode", mode = "auto" } }
local TURN_OFF = { PURIFIER, { command = "turn_off" } }

local LAMPS_ON = {
	{ FLOOR_LAMP, { state = "ON" } },
	{ TABLE_LAMP, { state = "ON" } },
}

local LAMPS_OFF = {
	{ FLOOR_LAMP, { state = "OFF" } },
	{ TABLE_LAMP, { state = "OFF" } },
}

local function stub(options)
	workflow = {
		step = function(_, run)
			return run()
		end,
	}

	sun = {
		is = test.spy(function(period)
			return period == options.period
		end),
	}

	presence = {
		at = function()
			return options.present == true
		end,
	}

	light = {
		is_on = function(device)
			return (options.lit or {})[device] == true
		end,
		set = test.spy(),
	}

	air_purifier = {
		state = function()
			return options.purifier
		end,
		command = test.spy(),
	}
end

---@type LuaTestSuite
return {
	["occupied at night turns the lamps and the purifier on"] = function()
		stub({ period = "night", purifier = { on = false } })

		living_room.occupied({ sunset_offset = "-30m" })

		test.eq(sun.is.calls, { { "night", "-30m" } })
		test.eq(light.set.calls, LAMPS_ON)
		test.eq(air_purifier.command.calls, { AUTO })
	end,

	["occupied by day only turns the purifier on"] = function()
		stub({ period = "day", purifier = { on = false } })

		living_room.occupied({ sunset_offset = "-30m" })

		test.eq(light.set.calls, {})
		test.eq(air_purifier.command.calls, { AUTO })
	end,

	["occupied leaves a running purifier in its mode"] = function()
		stub({ period = "day", purifier = { on = true, mode = "sleep" } })

		living_room.occupied({ sunset_offset = "-30m" })

		test.eq(air_purifier.command.calls, {})
	end,

	["occupied turns on a purifier that has never reported"] = function()
		stub({ period = "day" })

		living_room.occupied({ sunset_offset = "-30m" })

		test.eq(air_purifier.command.calls, { AUTO })
	end,

	["vacated turns the lamps and the purifier off"] = function()
		stub({ purifier = { on = true, mode = "auto" } })

		living_room.vacated()

		test.eq(light.set.calls, LAMPS_OFF)
		test.eq(air_purifier.command.calls, { TURN_OFF })
	end,

	["vacated leaves a stopped purifier alone"] = function()
		stub({ purifier = { on = false } })

		living_room.vacated()

		test.eq(light.set.calls, LAMPS_OFF)
		test.eq(air_purifier.command.calls, {})
	end,

	["sunset fades the floor lamp up and turns the table lamp on while occupied"] = function()
		stub({ present = true })

		living_room.sunset({ fade_rate = 1 })

		test.eq(light.set.calls, {
			{ FLOOR_LAMP, { state = "SET_BRIGHTNESS", value = 1 } },
			{ FLOOR_LAMP, { state = "INCREASE_BRIGHTNESS", value = 1 } },
			{ TABLE_LAMP, { state = "ON" } },
		})
		test.eq(air_purifier.command.calls, {})
	end,

	["sunset leaves lit lamps alone"] = function()
		stub({ present = true, lit = { [FLOOR_LAMP] = true, [TABLE_LAMP] = true } })

		living_room.sunset({ fade_rate = 1 })

		test.eq(light.set.calls, {})
	end,

	["sunset does nothing in an empty room"] = function()
		stub({ present = false })

		living_room.sunset({ fade_rate = 1 })

		test.eq(light.set.calls, {})
	end,
}
