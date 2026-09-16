---@type ZigbeeModel
return {
	roles = { "battery", "door" },

	decode = function(payload)
		return {
			battery = payload.battery,
			door = { contact = payload.contact },
			metrics = {
				device_temperature = payload.device_temperature,
				voltage = payload.voltage,
			},
		}
	end,
}
