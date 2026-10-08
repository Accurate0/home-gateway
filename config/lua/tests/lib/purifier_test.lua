local purifier = test.load("lib/purifier")

local DEVICE = "living-room-purifier"
local KEY = "purifier_quiet:" .. DEVICE
local NOW = 1000000

local function stub(current, held)
	local store = { [KEY] = held }

	state = {
		get = function(key)
			return store[key]
		end,
		set = function(key, value)
			store[key] = value
		end,
		clear = function(key)
			store[key] = nil
		end,
	}

	time = {
		now = function()
			return { epoch = NOW }
		end,
	}

	air_purifier = {
		state = function()
			return current
		end,
		command = test.spy(),
	}

	return store
end

local SLEEP = { DEVICE, { command = "set_mode", mode = "sleep" } }
local DISPLAY_OFF = { DEVICE, { command = "set_display", on = false } }

---@type LuaTestSuite
return {
	["a hold remembers what was wanted and quiets a running purifier"] = function()
		local store = stub({ on = true, mode = "manual", speed = 3, display = true })

		purifier.hold(DEVICE)

		test.eq(store[KEY], { since = NOW, mode = "manual", speed = 3, display = true })
		test.eq(air_purifier.command.calls, { SLEEP, DISPLAY_OFF })
	end,

	["a hold never turns the purifier on"] = function()
		local store = stub({ on = false, mode = "auto", display = true })

		purifier.hold(DEVICE)

		test.eq(store[KEY].mode, "auto")
		test.eq(air_purifier.command.calls, {})
	end,

	["a second hold keeps the first wanted settings"] = function()
		local held = { since = NOW - 60, mode = "auto", display = true }
		local store = stub({ on = true, mode = "sleep", display = false }, held)

		purifier.hold(DEVICE)

		test.eq(store[KEY], held)
		test.eq(air_purifier.command.calls, {})
	end,

	["without a hold a purifier event is left alone"] = function()
		stub(nil)

		purifier.enforce(DEVICE, { on = true, mode = "manual", speed = 3, display = true }, 6)

		test.eq(air_purifier.command.calls, {})
	end,

	["a change during a hold is remembered and quieted again"] = function()
		local store = stub(nil, { since = NOW - 60, mode = "auto", display = false })

		purifier.enforce(DEVICE, { on = true, mode = "manual", speed = 3, display = false }, 6)

		test.eq(store[KEY], { since = NOW - 60, mode = "manual", speed = 3, display = false })
		test.eq(air_purifier.command.calls, { SLEEP })
	end,

	["the event from its own quieting changes nothing"] = function()
		local held = { since = NOW - 60, mode = "auto", display = true }
		local store = stub(nil, held)

		purifier.enforce(DEVICE, { on = true, mode = "sleep", display = false }, 6)

		test.eq(store[KEY], held)
		test.eq(air_purifier.command.calls, {})
	end,

	["a purifier turned off during a hold stays off"] = function()
		stub(nil, { since = NOW - 60, mode = "auto" })

		purifier.enforce(DEVICE, { on = false, mode = "auto", display = true }, 6)

		test.eq(air_purifier.command.calls, {})
	end,

	["a stale hold is cleared instead of enforced"] = function()
		local store = stub(nil, { since = NOW - 7 * 60 * 60, mode = "auto" })

		purifier.enforce(DEVICE, { on = true, mode = "manual", speed = 3 }, 6)

		test.eq(store[KEY], nil)
		test.eq(air_purifier.command.calls, {})
	end,

	["a release restores a manual speed and the display"] = function()
		local store = stub(
			{ on = true, mode = "sleep", display = false },
			{ since = NOW - 60, mode = "manual", speed = 3, display = true }
		)

		purifier.release(DEVICE)

		test.eq(store[KEY], nil)
		test.eq(air_purifier.command.calls, {
			{ DEVICE, { command = "set_speed", speed = 3 } },
			{ DEVICE, { command = "set_display", on = true } },
		})
	end,

	["a release restores a mode and leaves a dark display dark"] = function()
		stub({ on = true, mode = "sleep", display = false }, { since = NOW - 60, mode = "auto", display = false })

		purifier.release(DEVICE)

		test.eq(air_purifier.command.calls, { { DEVICE, { command = "set_mode", mode = "auto" } } })
	end,

	["a release does nothing to a purifier that is off"] = function()
		local store = stub({ on = false, mode = "sleep" }, { since = NOW - 60, mode = "auto", display = true })

		purifier.release(DEVICE)

		test.eq(store[KEY], nil)
		test.eq(air_purifier.command.calls, {})
	end,
}
