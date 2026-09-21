---@type ZigbeeModel
return {
	roles = { "light" },
	capabilities = { "brightness", "colour_temp", "rgb" },

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
