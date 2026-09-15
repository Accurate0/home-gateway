---@type ZigbeeModel
return {
	roles = { "battery", "control_switch" },

	decode = function(payload)
		return {
			battery = payload.battery,
			control_switch = { action = payload.action },
			metrics = { voltage = payload.voltage },
		}
	end,
}
