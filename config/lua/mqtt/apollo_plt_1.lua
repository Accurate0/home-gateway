local ENVIRONMENT = {
	air_temperature = "temperature",
	air_humidity = "humidity",
	ltr390_light = "lux",
	ltr390_uv_index = "uv_index",
}

---@type EsphomeModel
return {
	protocol = "esphome",
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

	decode = function(input)
		local object_id = input.vars.object_id

		if object_id == "soil_moisture" then
			return { plant = { soil_moisture = input.payload } }
		end

		local metric = ENVIRONMENT[object_id]

		if metric then
			return { environment = { [metric] = input.payload } }
		end

		return { metrics = { [object_id] = input.payload } }
	end,
}
