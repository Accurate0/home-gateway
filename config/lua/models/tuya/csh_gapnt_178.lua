local COMMAND = "101"
local STATE = "102"
local BATTERY = "104"
local CONTACT = "105"

local STATES = {
	openning = "opening",
	opening = "opening",
	opened = "open",
	closing = "closing",
	closed = "closed",
}

local COMMANDS = {
	open = "fopen",
	close = "fclose",
}

local SETTLED = {
	open = "open",
	close = "closed",
}

---@type TuyaModel
return {
	roles = { "garage_door", "battery" },
	watchdog = "7d",

	encode = {
		garage_door = function(input)
			if input.current == SETTLED[input.command] then
				return nil
			end

			return { [COMMAND] = COMMANDS[input.command] }
		end,
	},

	decode = function(input)
		local dps = input.dps
		local changed = input.changed

		local reading = { battery = changed[BATTERY] }

		if changed[STATE] ~= nil or changed[CONTACT] ~= nil then
			reading.garage_door = {
				state = STATES[dps[STATE]],
				contact = dps[CONTACT],
			}
		end

		return reading
	end,
}
