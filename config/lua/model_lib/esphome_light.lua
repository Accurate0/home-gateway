local function rgb(hex)
	local value = tonumber((hex:gsub("^#", "")), 16)

	if value == nil then
		return nil
	end

	return {
		r = (value >> 16) & 0xff,
		g = (value >> 8) & 0xff,
		b = value & 0xff,
	}
end

---@type LightEncoder
return function(input)
	local command = input.command

	if command.type == "toggle" then
		return { state = input.current.on and "OFF" or "ON" }
	end

	if command.type ~= "set" or command.colour_temp ~= nil then
		return nil
	end

	local payload = {}

	if command.on ~= nil then
		payload.state = command.on and "ON" or "OFF"
	end

	payload.brightness = command.brightness

	if command.colour ~= nil then
		local colour = rgb(command.colour)

		if colour == nil then
			return nil
		end

		payload.color = colour
	end

	return payload
end
