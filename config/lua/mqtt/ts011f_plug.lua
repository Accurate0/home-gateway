---@type ZigbeeModel
return {
	protocol = "zigbee",
	roles = { "smart_switch" },

	encode = { smart_switch = gw.lib("zigbee_switch") },

	decode = function(input)
		local payload = input.payload

		return {
			smart_switch = {
				state = payload.state,
				voltage = payload.voltage,
				power = payload.power,
				current = payload.current,
				energy = payload.energy,
			},
		}
	end,
}
