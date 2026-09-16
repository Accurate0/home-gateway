---@type ZigbeeModel
return {
	roles = { "presence" },

	decode = function(payload)
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
