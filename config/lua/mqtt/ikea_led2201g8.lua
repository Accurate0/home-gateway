---@type ZigbeeModel
return {
	protocol = "zigbee",
	roles = { "light" },
	watchdog = "7d",
	capabilities = { "brightness", "colour_temp" },
	ranges = { colour_temp = { min = 250, max = 454 } },

	encode = { light = gw.lib("zigbee_light") },

	decode = function(input)
		local payload = input.payload

		return {
			light = {
				state = payload.state,
				brightness = payload.brightness,
				color_temp = payload.color_temp,
				color = payload.color,
			},
		}
	end,
}
