local purifier = {}

local QUIET_MODE = "sleep"
local RESTORE_MODE = "auto"
local HOUR = 60 * 60

function purifier.quiet(device, max_hours)
	local lock = device_lock.state(device)

	if lock ~= nil then
		gw.log(device .. " is already locked (" .. lock.source .. "), leaving it")
		return
	end

	local current = air_purifier.state(device)

	if current ~= nil and current.on == true then
		if current.mode ~= QUIET_MODE then
			air_purifier.command(device, { command = "set_mode", mode = QUIET_MODE })
		end

		if current.display ~= false then
			air_purifier.command(device, { command = "set_display", on = false })
		end
	end

	device_lock.lock(device, max_hours * HOUR)
end

function purifier.release(device)
	local lock = device_lock.state(device)

	if lock == nil then
		gw.log(device .. " is not locked, leaving it")
		return
	end

	if lock.source ~= "manual" then
		gw.log(device .. " is locked (" .. lock.source .. "), leaving it")
		return
	end

	device_lock.unlock(device)

	local current = air_purifier.state(device)

	if current == nil or current.on ~= true then
		return
	end

	air_purifier.command(device, { command = "set_mode", mode = RESTORE_MODE })
end

return purifier
