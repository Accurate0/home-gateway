---@type ZigbeeModel
return {
	roles = { "presence" },

	decode = function(payload)
		return {
			presence = { presence = payload.presence },
			metrics = {
				target_distance = payload.target_distance,
				movement = payload.movement,
			},
		}
	end,
}
