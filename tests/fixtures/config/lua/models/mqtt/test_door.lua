---@type ZigbeeModel
return {
	protocol = "zigbee",
	roles = { "battery", "door" },

	decode = function(input)
		local payload = input.payload

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
