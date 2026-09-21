---@type ZigbeeModel
return {
	protocol = "zigbee",
	roles = { "battery", "control_switch" },

	decode = function(input)
		local payload = input.payload

		return {
			battery = payload.battery,
			control_switch = { action = payload.action },
			metrics = { voltage = payload.voltage },
		}
	end,
}
