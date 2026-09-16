---@type ZigbeeModel
return {
	roles = { "battery", "environment" },
	environment = { "temperature", "humidity", "pressure" },

	decode = function(payload)
		return {
			battery = payload.battery,
			environment = {
				temperature = payload.temperature,
				humidity = payload.humidity,
				pressure = payload.pressure,
			},
		}
	end,
}
