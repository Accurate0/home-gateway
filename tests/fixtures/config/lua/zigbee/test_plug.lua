---@type ZigbeeModel
return {
	roles = { "smart_switch" },

	decode = function(payload)
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
