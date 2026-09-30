local model = test.load("models/mqtt/aqara_wxkg11lm")

---@type LuaTestSuite
return {
	["declares the button roles"] = function()
		test.eq(model.protocol, "zigbee")
		test.eq(model.roles, { "battery", "control_switch" })
	end,

	["decodes an action with battery and voltage"] = function()
		local reading = model.decode({
			topic = "state",
			vars = {},
			payload = { action = "double", battery = 87, voltage = 3005 },
		})

		test.eq(reading, {
			battery = 87,
			control_switch = { action = "double" },
			metrics = { voltage = 3005 },
		})
	end,

	["a report without an action leaves the action nil"] = function()
		local reading = model.decode({ topic = "state", vars = {}, payload = { battery = 50 } })

		test.eq(reading.control_switch, {})
	end,
}
