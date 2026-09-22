---@type ZigbeeModel
return {
	protocol = "zigbee",
	roles = { "light" },
	watchdog = "24h",
	capabilities = { "brightness", "colour_temp" },
	ranges = { colour_temp = { min = 153, max = 370 } },

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
