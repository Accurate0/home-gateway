local model = test.load("models/esphome_native_api/apollo_mtr_1")

local function decode(domain, object_id, payload)
	return model.decode({ domain = domain, object_id = object_id, payload = payload, entities = {} })
end

---@type LuaTestSuite
return {
	["declares the light, presence and environment roles"] = function()
		test.eq(model.roles, { "light", "presence", "environment" })
	end,

	["decodes the environment sensors"] = function()
		test.eq(decode("sensor", "dps310_temperature", 21.5), { environment = { temperature = 21.5 } })
		test.eq(decode("sensor", "dps310_pressure", 1012), { environment = { pressure = 1012 } })
		test.eq(decode("sensor", "ltr390_light", 140), { environment = { lux = 140 } })
	end,

	["decodes each radar binary sensor as its own presence source"] = function()
		test.eq(
			decode("binary_sensor", "ld2450_moving_target", true),
			{ presence = { presence = true, sensor = "ld2450_moving_target" } }
		)
	end,

	["decodes the rgb light"] = function()
		local light = { state = "ON", brightness = 128 }

		test.eq(decode("light", "rgb_light", light), { light = light })
	end,

	["ignores the entities the gateway does not use"] = function()
		test.eq(decode("sensor", "target-1_x", 412), {})
		test.eq(decode("binary_sensor", "online", true), {})
		test.eq(decode("sensor", "scd40_temperature", 22), {})
	end,
}
