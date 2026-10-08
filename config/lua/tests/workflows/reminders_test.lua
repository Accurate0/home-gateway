local reminders = test.load("workflows/reminders")

local SETTINGS = { device = "living-room-purifier", name = "Living Room Purifier", below = 10 }

local function stub(state)
	workflow = {
		step = function(_, run)
			return run()
		end,
	}

	air_purifier = {
		state = function()
			return state
		end,
	}

	notify = { send = test.spy() }
end

---@type LuaTestSuite
return {
	["a filter below the threshold sends a reminder"] = function()
		stub({ on = true, filter_life = 7.6 })

		reminders.filter_low(SETTINGS)

		test.eq(notify.send.calls, {
			{
				{
					title = "Living Room Purifier",
					message = "Living Room Purifier filter is at 8%, time to replace it",
					category = "general",
					tag = "filter_low:living-room-purifier",
				},
			},
		})
	end,

	["a healthy filter stays quiet"] = function()
		stub({ on = true, filter_life = 10 })

		reminders.filter_low(SETTINGS)

		test.eq(notify.send.calls, {})
	end,

	["a purifier with no filter reading stays quiet"] = function()
		stub({ on = true })
		reminders.filter_low(SETTINGS)
		test.eq(notify.send.calls, {})

		stub(nil)
		reminders.filter_low(SETTINGS)
		test.eq(notify.send.calls, {})
	end,
}
