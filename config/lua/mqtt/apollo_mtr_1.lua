local ENVIRONMENT = {
	dps310_temperature = "temperature",
	dps310_pressure = "pressure",
	ltr390_light = "lux",
}

---@type EsphomeModel
return {
	protocol = "esphome",
	roles = { "light", "presence", "environment" },
	capabilities = { "brightness", "rgb", "temperature", "pressure", "lux" },

	entities = {
		sensor = { "dps310_temperature", "dps310_pressure", "ltr390_light" },
		binary_sensor = { "ld2450_presence", "ld2450_moving_target", "ld2450_still_target" },
		light = { "rgb_light" },
	},

	encode = { light = gw.lib("esphome_light") },

	decode = function(input)
		local entity = input.vars

		if entity.domain == "light" then
			return { light = input.payload }
		end

		if entity.domain == "binary_sensor" then
			return { presence = { presence = input.payload, sensor = entity.object_id } }
		end

		local metric = ENVIRONMENT[entity.object_id]

		if metric then
			return { environment = { [metric] = input.payload } }
		end

		return { metrics = { [entity.object_id] = input.payload } }
	end,
}
