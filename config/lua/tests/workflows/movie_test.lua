local movie = test.load("workflows/movie")

local FLOOR_LAMP = "floor-lamp-living-room"
local TABLE_LAMP = "living-room-table-lamp"
local OCCUPANCY_TAG = "living-room-occupancy"

local SETTINGS = { pause_brightness = 190, fade_rate = 13, sunset_offset = "-30m" }

local function stub(options)
	event = { state = options.state, item = "Film" }

	workflow = {
		step = function(_, run)
			return run()
		end,
		set_enabled = test.spy(),
	}

	sun = {
		is = test.spy(function(period)
			return period == options.period
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
	["started turns the lamps off and pauses occupancy"] = function()
		stub({ state = "started", period = "day" })

		movie.playback(SETTINGS)

		test.eq(workflow.set_enabled.calls, { { OCCUPANCY_TAG, "DISABLED" } })
		test.eq(light.set.calls, {
			{ FLOOR_LAMP, { state = "OFF" } },
			{ TABLE_LAMP, { state = "OFF" } },
		})
	end,

	["paused at night raises the floor lamp"] = function()
		stub({ state = "paused", period = "night" })

		movie.playback(SETTINGS)

		test.eq(sun.is.calls, { { "night", "-30m" } })
		test.eq(light.set.calls, { { FLOOR_LAMP, { state = "SET_BRIGHTNESS", value = 190 } } })
	end,

	["paused by day leaves the lamps off"] = function()
		stub({ state = "paused", period = "day" })

		movie.playback(SETTINGS)

		test.eq(light.set.calls, {})
	end,

	["stopped at night fades the lamps up"] = function()
		stub({ state = "stopped", period = "night" })

		movie.playback(SETTINGS)

		test.eq(workflow.set_enabled.calls, { { OCCUPANCY_TAG, "ENABLED" } })
		test.eq(light.set.calls, {
			{ FLOOR_LAMP, { state = "SET_BRIGHTNESS", value = 1 } },
			{ FLOOR_LAMP, { state = "INCREASE_BRIGHTNESS", value = 13 } },
			{ TABLE_LAMP, { state = "ON" } },
		})
	end,

	["stopped by day restores occupancy and leaves the lamps off"] = function()
		stub({ state = "stopped", period = "day" })

		movie.playback(SETTINGS)

		test.eq(workflow.set_enabled.calls, { { OCCUPANCY_TAG, "ENABLED" } })
		test.eq(light.set.calls, {})
	end,
}
