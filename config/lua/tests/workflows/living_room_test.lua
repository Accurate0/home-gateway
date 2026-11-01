local living_room = test.load("workflows/living_room")

local FLOOR_LAMP = "floor-lamp-living-room"
local TABLE_LAMP = "living-room-table-lamp"

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
		seen_within = test.spy(function()
			return options.present == true
		end),
	}

	light = {
		is_on = function(device)
			return (options.lit or {})[device] == true
		end,
		set = test.spy(),
	}
end

---@type LuaTestSuite
return {
	["occupied at night turns the lamps on"] = function()
		stub({ period = "night" })

		living_room.occupied({ sunset_offset = "-30m" })

		test.eq(sun.is.calls, { { "night", "-30m" } })
		test.eq(light.set.calls, LAMPS_ON)
	end,

	["occupied by day leaves the lamps off"] = function()
		stub({ period = "day" })

		living_room.occupied({ sunset_offset = "-30m" })

		test.eq(light.set.calls, {})
	end,

	["vacated turns the lamps off"] = function()
		stub({})

		living_room.vacated()

		test.eq(light.set.calls, LAMPS_OFF)
	end,

	["sunset fades the floor lamp up and turns the table lamp on while occupied"] = function()
		stub({ present = true })

		living_room.sunset({ fade_rate = 1, vacancy = "25m" })

		test.eq(presence.seen_within.calls, { { "livingroom-motion", "25m" } })
		test.eq(light.set.calls, {
			{ FLOOR_LAMP, { state = "SET_BRIGHTNESS", value = 1 } },
			{ FLOOR_LAMP, { state = "INCREASE_BRIGHTNESS", value = 1 } },
			{ TABLE_LAMP, { state = "ON" } },
		})
	end,

	["sunset leaves lit lamps alone"] = function()
		stub({ present = true, lit = { [FLOOR_LAMP] = true, [TABLE_LAMP] = true } })

		living_room.sunset({ fade_rate = 1, vacancy = "25m" })

		test.eq(light.set.calls, {})
	end,

	["sunset does nothing in an empty room"] = function()
		stub({ present = false })

		living_room.sunset({ fade_rate = 1, vacancy = "25m" })

		test.eq(light.set.calls, {})
	end,
}
