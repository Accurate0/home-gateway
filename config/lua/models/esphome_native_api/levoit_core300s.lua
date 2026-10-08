local FAN = "fan"
local PM25 = "aq_-_pm_2_5"
local FILTER_LIFE = "filter__"
local DISPLAY = "display"

local MAX_SPEED = 3

local MODES = {
	Manual = "manual",
	Sleep = "sleep",
	Auto = "auto",
}

local PRESETS = {
	manual = "Manual",
	sleep = "Sleep",
	auto = "Auto",
}

local PURIFIER = {
	[FAN] = true,
	[PM25] = true,
	[FILTER_LIFE] = true,
	[DISPLAY] = true,
}

local COMMANDS = {
	turn_on = function(_, current)
		if current.on == true then
			return nil
		end

		return { state = "ON" }
	end,

	turn_off = function(_, current)
		if current.on == false then
			return nil
		end

		return { state = "OFF" }
	end,

	set_mode = function(command, current)
		if current.on == true and current.mode == command.mode then
			return nil
		end

		return { state = "ON", preset = PRESETS[command.mode] }
	end,

	set_speed = function(command, current)
		local speed = command.speed

		if speed < 1 or speed > MAX_SPEED then
			error("speed must be between 1 and " .. MAX_SPEED .. ", got " .. speed)
		end

		if current.on == true and current.mode == "manual" and current.speed == speed then
			return nil
		end

		return { state = "ON", preset = PRESETS.manual, speed = speed }
	end,

	set_display = function(command, current)
		if current.display == command.on then
			return nil
		end

		return { switches = { [DISPLAY] = command.on } }
	end,
}

---@type EsphomeNativeApiModel
return {
	roles = { "air_purifier" },
	watchdog = "1h",

	encode = {
		air_purifier = function(input)
			local command = input.command
			local encode = COMMANDS[command.command]

			if encode == nil then
				error("unknown air purifier command " .. tostring(command.command))
			end

			return encode(command, input.current)
		end,
	},

	decode = function(input)
		if not PURIFIER[input.object_id] then
			return { metrics = { [input.object_id] = input.payload } }
		end

		local entities = input.entities
		local fan = entities[FAN]

		if fan == nil then
			return {}
		end

		return {
			air_purifier = {
				state = fan.state,
				mode = MODES[fan.preset],
				speed = fan.speed,
				pm25 = entities[PM25],
				filter_life = entities[FILTER_LIFE],
				display = entities[DISPLAY],
			},
		}
	end,
}
