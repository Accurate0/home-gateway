---@type ZigbeeModel
return {
	roles = { "environment" },
	environment = { "temperature", "humidity", "pm25", "voc_index" },

	decode = function(payload)
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
