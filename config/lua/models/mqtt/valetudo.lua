---@type ValetudoModel
return {
	protocol = "valetudo",
	roles = { "robot_vacuum", "battery" },
	watchdog = "7d",

	commands = {
		robot_vacuum = { start = "start", stop = "stop", dock = "return_to_base" },
	},

	decode = function(input)
		local payload = input.payload

		if type(payload) ~= "table" then
			return {}
		end

		if input.topic == "attributes" then
			return {
				robot_vacuum = {
					clean_area = tonumber(payload.currentCleanArea),
					clean_count = payload.cleanCount,
					attributes = payload,
				},
			}
		end

		return {
			robot_vacuum = {
				status = payload.state,
				battery = payload.battery_level,
				fan_speed = payload.fan_speed,
			},
		}
	end,
}
