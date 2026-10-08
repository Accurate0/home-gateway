local model = test.load("models/esphome_native_api/levoit_core300s")

local function encode(command, current)
	return model.encode.air_purifier({ command = command, current = current or {} })
end

---@type LuaTestSuite
return {
	["declares the air purifier role"] = function()
		test.eq(model.roles, { "air_purifier" })
	end,

	["decodes the fan with the sensors seen so far"] = function()
		local fan = { state = "ON", speed = 2, preset = "Manual" }

		local reading = model.decode({
			domain = "sensor",
			object_id = "aq_-_pm_2_5",
			payload = 4,
			entities = { fan = fan, ["aq_-_pm_2_5"] = 4, ["filter__"] = 87, display = true },
		})

		test.eq(reading, {
			air_purifier = {
				state = "ON",
				mode = "manual",
				speed = 2,
				pm25 = 4,
				filter_life = 87,
				display = true,
			},
		})
	end,

	["a sensor before the fan state is not a reading"] = function()
		local reading = model.decode({
			domain = "sensor",
			object_id = "aq_-_pm_2_5",
			payload = 4,
			entities = { ["aq_-_pm_2_5"] = 4 },
		})

		test.eq(reading, {})
	end,

	["the air quality sensors land as metrics"] = function()
		test.eq(model.decode({ domain = "sensor", object_id = "aqi", payload = 1, entities = { aqi = 1 } }), {
			metrics = { aqi = 1 },
		})
		test.eq(model.decode({ domain = "sensor", object_id = "current_cadr", payload = 120, entities = {} }), {
			metrics = { cadr = 120 },
		})
	end,

	["diagnostics and unnamed entities are dropped"] = function()
		local fan = { state = "ON", speed = 1 }

		test.eq(model.decode({ domain = "sensor", object_id = "heap_free", payload = 1024, entities = {} }), {})
		test.eq(model.decode({ domain = "fan", object_id = "", payload = fan, entities = { [""] = fan } }), {})
	end,

	["power commands only go out when they change something"] = function()
		test.eq(encode({ command = "turn_on" }, { on = false }), { state = "ON" })
		test.eq(encode({ command = "turn_on" }, { on = true }), nil)
		test.eq(encode({ command = "turn_off" }, { on = true }), { state = "OFF" })
		test.eq(encode({ command = "turn_off" }, { on = false }), nil)
		test.eq(encode({ command = "turn_on" }), { state = "ON" })
	end,

	["a mode turns the purifier on with its preset"] = function()
		test.eq(encode({ command = "set_mode", mode = "auto" }, { on = false, mode = "auto" }), {
			state = "ON",
			preset = "Auto",
		})
		test.eq(encode({ command = "set_mode", mode = "sleep" }, { on = true, mode = "sleep" }), nil)
	end,

	["a speed implies manual mode and stays in range"] = function()
		test.eq(encode({ command = "set_speed", speed = 3 }, { on = true, mode = "auto", speed = 1 }), {
			state = "ON",
			preset = "Manual",
			speed = 3,
		})
		test.eq(encode({ command = "set_speed", speed = 2 }, { on = true, mode = "manual", speed = 2 }), nil)

		test.errors(function()
			encode({ command = "set_speed", speed = 4 })
		end, "between 1 and 3")
	end,

	["the display is a switch command that leaves the fan alone"] = function()
		test.eq(encode({ command = "set_display", on = false }, { on = true, display = true }), {
			switches = { display = false },
		})
		test.eq(encode({ command = "set_display", on = false }, { display = false }), nil)
	end,
}
