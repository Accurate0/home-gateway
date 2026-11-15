local purifier = {}

local function set_mode(device, mode)
	workflow.step("mode_" .. mode, function()
		local current = air_purifier.state(device)

		if current ~= nil and current.on == true and current.mode == mode then
			gw.log(device .. " is already on " .. mode)
			return
		end

		air_purifier.command(device, { command = "set_mode", mode = mode })
	end, device)
end

local function set_display(device, on)
	workflow.step(on and "display_on" or "display_off", function()
		local current = air_purifier.state(device)

		if current ~= nil and current.display == on then
			gw.log(device .. " display is already " .. (on and "on" or "off"))
			return
		end

		air_purifier.command(device, { command = "set_display", on = on })
	end, device)
end

function purifier.auto(settings)
	set_mode(settings.device, "auto")
end

function purifier.night(settings)
	set_mode(settings.device, "sleep")
	set_display(settings.device, false)
end

function purifier.morning(settings)
	set_mode(settings.device, "auto")
	set_display(settings.device, true)
end

function purifier.off_once_today(settings)
	local key = "purifier_off_once:" .. settings.device
	local today = time.now().date

	if state.get(key) == today then
		gw.log(settings.device .. " was already turned off today")
		return
	end

	workflow.step("off", function()
		local current = air_purifier.state(settings.device)

		if current ~= nil and current.on == false then
			gw.log(settings.device .. " is already off")
			return
		end

		air_purifier.command(settings.device, { command = "turn_off" })
	end, settings.device)

	state.set(key, today)
end

return purifier
