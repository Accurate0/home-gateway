---@type ZigbeeModel
return {
	protocol = "zigbee",
	roles = { "light" },
	capabilities = { "brightness", "colour_temp" },

	decode = function(input)
		local payload = input.payload

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
