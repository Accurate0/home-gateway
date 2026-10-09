local ENVIRONMENT = {
	dps310_temperature = "temperature",
	dps310_pressure = "pressure",
	ltr390_light = "lux",
}

local PRESENCE = {
	ld2450_presence = true,
	ld2450_moving_target = true,
	ld2450_still_target = true,
}

---@type EsphomeNativeApiModel
return {
	roles = { "light", "presence", "environment" },
	watchdog = "1h",
	capabilities = { "brightness", "rgb", "temperature", "pressure", "lux" },
	ranges = { brightness = { min = 0, max = 255 } },

	encode = { light = gw.lib("esphome_light") },

	decode = function(input)
		if input.domain == "light" then
			return { light = input.payload }
		end

		if input.domain == "binary_sensor" and PRESENCE[input.object_id] then
			return { presence = { presence = input.payload, sensor = input.object_id } }
		end

		local metric = input.domain == "sensor" and ENVIRONMENT[input.object_id]

		if metric then
			return { environment = { [metric] = input.payload } }
		end

		return {}
	end,
}
