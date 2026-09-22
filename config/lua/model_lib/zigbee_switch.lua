---@type LightEncoder
return function(input)
	local command = input.command

	if command.type == "toggle" then
		return { state = "TOGGLE" }
	end

	if command.type ~= "set" or command.on == nil then
		return nil
	end

	return { state = command.on and "ON" or "OFF" }
end
