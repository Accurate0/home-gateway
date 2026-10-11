local small_switch = test.load("workflows/small_switch")

local FLOOR_LAMP = "floor-lamp-living-room"
local TABLE_LAMP = "living-room-table-lamp"
local INDICATOR = "livingroom-motion"

local function stub(action, on)
	event = { action = action }

	workflow = {
		step = function(_, run)
			return run()
		end,
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

		test.eq(light.set.calls[1], { FLOOR_LAMP, { state = "OFF" } })
		test.eq(light.set.calls[2], { TABLE_LAMP, { state = "OFF" } })
	end,

	["single tap turns the lamps on while the floor lamp is off"] = function()
		stub("single", {})

		small_switch.tap()

		test.eq(light.set.calls[1], { FLOOR_LAMP, { state = "ON" } })
		test.eq(light.set.calls[2], { TABLE_LAMP, { state = "ON" } })
	end,

	["double tap toggles the living room automations"] = function()
		stub("double", {})

		small_switch.tap()

		test.eq(workflow.set_enabled.calls, { { "living-room", "TOGGLE" } })
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
			{ FLOOR_LAMP, { state = "ON" } },
			{ TABLE_LAMP, { state = "ON" } },
			{ INDICATOR, { state = "OFF" } },
			{ INDICATOR, { state = "ON" } },
		})
	end,

	["an unknown action does nothing"] = function()
		stub("hold", {})

		small_switch.tap()

		test.eq(light.set.calls, {})
	end,
}
