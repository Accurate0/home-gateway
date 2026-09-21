local ENVIRONMENT = {
	test_node_temperature = "temperature",
	test_node_humidity = "humidity",
}

---@type EsphomeModel
return {
	roles = { "environment" },
	capabilities = { "temperature", "humidity" },

	entities = {
		sensor = { "test_node_temperature", "test_node_humidity" },
	},

	decode = function(entity)
		local metric = ENVIRONMENT[entity.object_id]

		if metric then
			return { environment = { [metric] = entity.state } }
		end

		return { metrics = { [entity.object_id] = entity.state } }
	end,
}
