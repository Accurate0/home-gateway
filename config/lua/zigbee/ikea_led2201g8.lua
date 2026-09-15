---@type ZigbeeModel
return {
	roles = { "light" },

	decode = function(payload)
		return {
			light = {
				state = payload.state,
				brightness = payload.brightness,
				color_temp = payload.color_temp,
				color = payload.color,
			},
			metrics = {
				brightness = payload.brightness,
				color_temp = payload.color_temp,
			},
		}
	end,
}
