---@type ZigbeeModel
return {
	protocol = "zigbee",
	roles = { "battery", "environment" },
	watchdog = "3h",
	capabilities = { "temperature", "humidity", "pressure" },

	decode = function(input)
		local payload = input.payload

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
