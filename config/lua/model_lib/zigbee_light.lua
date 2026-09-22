---@type LightEncoder
return function(input)
	local command = input.command

	if command.type == "set" then
		local payload = {
			brightness = command.brightness,
			color_temp = command.colour_temp,
		}

		if command.on ~= nil then
			payload.state = command.on and "ON" or "OFF"
		end

		if command.colour ~= nil then
			payload.color = { hex = command.colour }
		end

		return payload
	end

	if command.type == "toggle" then
		return { state = "TOGGLE" }
	end

	if command.type == "brightness_move" then
		if command.on_off then
			return { brightness_move_onoff = command.value }
		end

		return { brightness_move = command.value }
	end

	if command.type == "colour_temp_move" then
		if command.value == 0 then
			return { color_temp_move = "stop" }
		end

		return { color_temp_move = command.value }
	end

	return nil
end
