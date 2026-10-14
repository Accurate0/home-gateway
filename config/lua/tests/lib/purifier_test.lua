local purifier = test.load("lib/purifier")

local DEVICE = "living-room-purifier"
local FOUR_HOURS = 4 * 60 * 60

local SLEEP = { DEVICE, { command = "set_mode", mode = "sleep" } }
local DISPLAY_OFF = { DEVICE, { command = "set_display", on = false } }
local AUTO = { DEVICE, { command = "set_mode", mode = "auto" } }

local function stub(current, lock)
	air_purifier = {
		state = function()
			return current
		end,
		command = test.spy(),
	}

	device_lock = {
		state = function()
			return lock
		end,
		lock = test.spy(),
		unlock = test.spy(),
	}
end

---@type LuaTestSuite
return {
	["quiet sets a running purifier to sleep with the display off, then locks it"] = function()
		stub({ on = true, mode = "auto", display = true })

		purifier.quiet(DEVICE, 4)

		test.eq(air_purifier.command.calls, { SLEEP, DISPLAY_OFF })
		test.eq(device_lock.lock.calls, { { DEVICE, FOUR_HOURS } })
	end,

	["quiet skips commands a purifier already satisfies"] = function()
		stub({ on = true, mode = "sleep", display = false })

		purifier.quiet(DEVICE, 4)

		test.eq(air_purifier.command.calls, {})
		test.eq(device_lock.lock.calls, { { DEVICE, FOUR_HOURS } })
	end,

	["quiet never turns the purifier on but still locks it"] = function()
		stub({ on = false, mode = "auto", display = true })

		purifier.quiet(DEVICE, 4)

		test.eq(air_purifier.command.calls, {})
		test.eq(device_lock.lock.calls, { { DEVICE, FOUR_HOURS } })
	end,

	["quiet leaves an existing manual lock and its expiry alone"] = function()
		stub({ on = true, mode = "sleep", display = false }, { source = "manual" })

		purifier.quiet(DEVICE, 4)

		test.eq(air_purifier.command.calls, {})
		test.eq(device_lock.lock.calls, {})
	end,

	["quiet respects a physical lock"] = function()
		stub({ on = true, mode = "manual", display = true }, { source = "physical" })

		purifier.quiet(DEVICE, 4)

		test.eq(air_purifier.command.calls, {})
		test.eq(device_lock.lock.calls, {})
	end,

	["release unlocks a manual lock and puts a running purifier back on auto"] = function()
		stub({ on = true, mode = "sleep", display = false }, { source = "manual" })

		purifier.release(DEVICE)

		test.eq(device_lock.unlock.calls, { { DEVICE } })
		test.eq(air_purifier.command.calls, { AUTO })
	end,

	["release unlocks but leaves a stopped purifier off"] = function()
		stub({ on = false, mode = "sleep" }, { source = "manual" })

		purifier.release(DEVICE)

		test.eq(device_lock.unlock.calls, { { DEVICE } })
		test.eq(air_purifier.command.calls, {})
	end,

	["release keeps a physical lock"] = function()
		stub({ on = true, mode = "manual", display = true }, { source = "physical" })

		purifier.release(DEVICE)

		test.eq(device_lock.unlock.calls, {})
		test.eq(air_purifier.command.calls, {})
	end,

	["release does nothing when unlocked"] = function()
		stub({ on = true, mode = "auto" })

		purifier.release(DEVICE)

		test.eq(device_lock.unlock.calls, {})
		test.eq(air_purifier.command.calls, {})
	end,
}
