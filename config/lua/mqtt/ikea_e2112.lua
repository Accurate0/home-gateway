---@type ZigbeeModel
return {
	protocol = "zigbee",
	roles = { "environment" },
	capabilities = { "temperature", "humidity", "pm25", "voc_index" },

	decode = function(input)
		local payload = input.payload

		return {
			environment = {
				temperature = payload.temperature,
				humidity = payload.humidity,
				pm25 = payload.pm25,
				voc_index = payload.voc_index,
			},
		}
	end,
}
