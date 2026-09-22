---@type ZigbeeModel
return {
	protocol = "zigbee",
	roles = { "smart_switch" },

	encode = {
		smart_switch = function(input)
			local command = input.command

			if command.type == "toggle" then
				return { state = "TOGGLE" }
			end

			if command.type == "brightness_move" then
				return { brightness_move = command.value }
			end

			if command.type ~= "set" then
				return nil
			end

			local payload = { brightness = command.brightness }

			if command.on ~= nil then
				payload.state = command.on and "ON" or "OFF"
			end

			return payload
		end,
	},

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
