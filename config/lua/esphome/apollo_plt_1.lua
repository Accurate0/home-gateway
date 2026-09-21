local ENVIRONMENT = {
	air_temperature = "temperature",
	air_humidity = "humidity",
	ltr390_light = "lux",
	ltr390_uv_index = "uv_index",
}

---@type EsphomeModel
return {
	roles = { "environment", "plant" },
	capabilities = { "temperature", "humidity", "lux", "uv_index" },
	plant = { "soil_moisture" },

	entities = {
		sensor = {
			"air_temperature",
			"air_humidity",
			"ltr390_light",
			"ltr390_uv_index",
			"soil_moisture",
		},
	},

	decode = function(entity)
		if entity.object_id == "soil_moisture" then
			return { plant = { soil_moisture = entity.state } }
		end

		local metric = ENVIRONMENT[entity.object_id]

		if metric then
			return { environment = { [metric] = entity.state } }
		end

		return { metrics = { [entity.object_id] = entity.state } }
	end,
}
