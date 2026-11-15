local purifier = test.load("workflows/purifier")

local PURIFIER = "bedroom-purifier"

local AUTO = { PURIFIER, { command = "set_mode", mode = "auto" } }
local SLEEP = { PURIFIER, { command = "set_mode", mode = "sleep" } }
local DISPLAY_ON = { PURIFIER, { command = "set_display", on = true } }
local DISPLAY_OFF = { PURIFIER, { command = "set_display", on = false } }
local OFF = { PURIFIER, { command = "turn_off" } }
local OFF_KEY = "purifier_off_once:" .. PURIFIER

local function stub(current)
	workflow = {
		step = function(_, run)
			return run()
		end,
	}

	air_purifier = {
		state = function()
			return current
		end,
		command = test.spy(),
	}
end

local function stub_today(stored)
	local values = { [OFF_KEY] = stored }

	time = {
		now = function()
			return { date = "2026-11-15" }
		end,
	}

	state = {
		get = function(key)
			return values[key]
		end,
		set = function(key, value)
			values[key] = value
		end,
	}

	return values
end

---@type LuaTestSuite
return {
	["auto turns on a stopped purifier"] = function()
		stub({ on = false })

		purifier.auto({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { AUTO })
	end,

	["auto switches a running purifier to auto"] = function()
		stub({ on = true, mode = "manual" })

		purifier.auto({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { AUTO })
	end,

	["auto leaves a purifier already on auto"] = function()
		stub({ on = true, mode = "auto" })

		purifier.auto({ device = PURIFIER })

		test.eq(air_purifier.command.calls, {})
	end,

	["auto turns on a purifier that has never reported"] = function()
		stub(nil)

		purifier.auto({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { AUTO })
	end,

	["night sets a running purifier to sleep with the display off"] = function()
		stub({ on = true, mode = "auto", display = true })

		purifier.night({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { SLEEP, DISPLAY_OFF })
	end,

	["night turns on a stopped purifier"] = function()
		stub({ on = false, mode = "sleep", display = false })

		purifier.night({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { SLEEP })
	end,

	["night commands a purifier that has never reported"] = function()
		stub(nil)

		purifier.night({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { SLEEP, DISPLAY_OFF })
	end,

	["night skips what a purifier already satisfies"] = function()
		stub({ on = true, mode = "sleep", display = false })

		purifier.night({ device = PURIFIER })

		test.eq(air_purifier.command.calls, {})
	end,

	["morning sets a running purifier to auto with the display on"] = function()
		stub({ on = true, mode = "sleep", display = false })

		purifier.morning({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { AUTO, DISPLAY_ON })
	end,

	["morning turns on a stopped purifier"] = function()
		stub({ on = false, mode = "auto", display = true })

		purifier.morning({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { AUTO })
	end,

	["morning commands a purifier that has never reported"] = function()
		stub(nil)

		purifier.morning({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { AUTO, DISPLAY_ON })
	end,

	["morning skips what a purifier already satisfies"] = function()
		stub({ on = true, mode = "auto", display = true })

		purifier.morning({ device = PURIFIER })

		test.eq(air_purifier.command.calls, {})
	end,

	["off_once_today turns off a running purifier and remembers the day"] = function()
		stub({ on = true, mode = "auto" })
		local values = stub_today(nil)

		purifier.off_once_today({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { OFF })
		test.eq(values[OFF_KEY], "2026-11-15")
	end,

	["off_once_today does nothing a second time the same day"] = function()
		stub({ on = true, mode = "auto" })
		stub_today("2026-11-15")

		purifier.off_once_today({ device = PURIFIER })

		test.eq(air_purifier.command.calls, {})
	end,

	["off_once_today runs again on a new day"] = function()
		stub({ on = true, mode = "auto" })
		stub_today("2026-11-14")

		purifier.off_once_today({ device = PURIFIER })

		test.eq(air_purifier.command.calls, { OFF })
	end,

	["off_once_today skips a purifier that is already off"] = function()
		stub({ on = false })
		local values = stub_today(nil)

		purifier.off_once_today({ device = PURIFIER })

		test.eq(air_purifier.command.calls, {})
		test.eq(values[OFF_KEY], "2026-11-15")
	end,
}
