local purifier = test.load("workflows/purifier")

local PURIFIER = "bedroom-purifier"

local AUTO = { PURIFIER, { command = "set_mode", mode = "auto" } }
local SLEEP = { PURIFIER, { command = "set_mode", mode = "sleep" } }
local DISPLAY_ON = { PURIFIER, { command = "set_display", on = true } }
local DISPLAY_OFF = { PURIFIER, { command = "set_display", on = false } }

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
}
