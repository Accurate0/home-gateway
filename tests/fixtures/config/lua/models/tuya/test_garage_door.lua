---@type TuyaModel
return {
	roles = { "garage_door", "battery" },
	watchdog = "1h",

	encode = {
		garage_door = function(input)
			return { ["1"] = input.command == "open" }
		end,
	},

	decode = function(input)
		return {
			battery = input.changed["3"],
			garage_door = { state = input.dps["2"] },
		}
	end,
}
