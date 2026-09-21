local ENVIRONMENT = {
	test_node_temperature = "temperature",
	test_node_humidity = "humidity",
}

---@type EsphomeModel
return {
	protocol = "esphome",
	roles = { "environment" },
	capabilities = { "temperature", "humidity" },

	entities = {
		sensor = { "test_node_temperature", "test_node_humidity" },
	},

	decode = function(input)
		local object_id = input.vars.object_id
		local metric = ENVIRONMENT[object_id]

		if metric then
			return { environment = { [metric] = input.payload } }
		end

		return { metrics = { [object_id] = input.payload } }
	end,
}
