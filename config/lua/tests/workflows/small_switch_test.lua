local small_switch = test.load("workflows/small_switch")

local FLOOR_LAMP = "floor-lamp-living-room"
local INDICATOR = "livingroom-motion"

local function stub(action, on)
	event = { action = action }

	workflow = {
		step = function(_, run)
			return run()
		end,
		run = test.spy(),
		set_enabled = test.spy(),
	}

	light = {
		is_on = function(device)
			return on[device] == true
		end,
		set = test.spy(),
	}
end

---@type LuaTestSuite
return {
	["single tap turns the lamps off while the floor lamp is on"] = function()
		stub("single", { [FLOOR_LAMP] = true })

		small_switch.tap()

		test.eq(workflow.run.calls, { { "living-room-lamps-off" } })
	end,

	["single tap turns the lamps on while the floor lamp is off"] = function()
		stub("single", {})

		small_switch.tap()

		test.eq(workflow.run.calls, { { "living-room-lamps-on" } })
	end,

	["double tap toggles the living room automations"] = function()
		stub("double", {})

		small_switch.tap()

		test.eq(workflow.set_enabled.calls, { { "living-room", "TOGGLE" } })
		test.eq(workflow.run.calls, {})
	end,

	["the indicator flashes once per tap and ends where it started"] = function()
		stub("triple", {})

		small_switch.tap()

		local expected = { { INDICATOR, { state = "SET_BRIGHTNESS", value = 128 } } }

		for _ = 1, 3 do
			table.insert(expected, { INDICATOR, { state = "ON" } })
			table.insert(expected, { INDICATOR, { state = "OFF" } })
		end

		test.eq(light.set.calls, expected)
	end,

	["a lit indicator blinks off without setting brightness"] = function()
		stub("single", { [INDICATOR] = true })

		small_switch.tap()

		test.eq(light.set.calls, {
			{ INDICATOR, { state = "OFF" } },
			{ INDICATOR, { state = "ON" } },
		})
	end,

	["an unknown action does nothing"] = function()
		stub("hold", {})

		small_switch.tap()

		test.eq(workflow.run.calls, {})
		test.eq(light.set.calls, {})
	end,
}
