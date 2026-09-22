---@type ZigbeeModel
return {
	protocol = "zigbee",
	roles = { "presence" },
	watchdog = "1h",

	decode = function(input)
		local payload = input.payload

		return {
			presence = { presence = payload.presence },
			metrics = {
				target_distance = payload.target_distance,
				device_temperature = payload.device_temperature,
				movement = payload.movement,
			},
		}
	end,
}
