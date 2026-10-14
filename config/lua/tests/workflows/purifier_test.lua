local purifier = test.load("workflows/purifier")

local PURIFIER = "living-room-purifier"

local AUTO = { PURIFIER, { command = "set_mode", mode = "auto" } }

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
}
