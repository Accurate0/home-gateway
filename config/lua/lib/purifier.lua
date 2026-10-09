local purifier = {}

local QUIET_MODE = "sleep"
local HOUR = 60 * 60

local function key(device)
	return "purifier_quiet:" .. device
end

local function quiet(device, current)
	if current == nil or current.on ~= true then
		return
	end

	if current.mode ~= QUIET_MODE then
		air_purifier.command(device, { command = "set_mode", mode = QUIET_MODE })
	end

	if current.display ~= false then
		air_purifier.command(device, { command = "set_display", on = false })
	end
end

function purifier.hold(device)
	gw.require("air_purifier:write")

	local current = air_purifier.state(device)

	if state.get(key(device)) == nil then
		local wanted = current or {}

		state.set(key(device), {
			since = time.now().epoch,
			mode = wanted.mode,
			speed = wanted.speed,
			display = wanted.display,
		})
	end

	quiet(device, current)
end

function purifier.enforce_quiet(device, current, max_hours)
	gw.require("air_purifier:write")

	local held = state.get(key(device))

	if held == nil then
		return
	end

	if time.now().epoch - held.since > max_hours * HOUR then
		gw.log("quiet hold on " .. device .. " is stale, clearing it")
		state.clear(key(device))
		return
	end

	if current.on ~= true then
		return
	end

	local changed = false

	if current.mode ~= nil and current.mode ~= QUIET_MODE then
		held.mode = current.mode
		held.speed = current.speed
		changed = true
	end

	if current.display == true then
		held.display = true
		changed = true
	end

	if not changed then
		return
	end

	state.set(key(device), held)
	quiet(device, current)
end

function purifier.release(device)
	gw.require("air_purifier:write")

	local held = state.get(key(device))

	if held == nil then
		return
	end

	state.clear(key(device))

	local current = air_purifier.state(device)

	if current == nil or current.on ~= true then
		return
	end

	if held.mode == "manual" and held.speed ~= nil and held.speed > 0 then
		air_purifier.command(device, { command = "set_speed", speed = held.speed })
	elseif held.mode ~= nil and held.mode ~= current.mode then
		air_purifier.command(device, { command = "set_mode", mode = held.mode })
	end

	if held.display == true then
		air_purifier.command(device, { command = "set_display", on = true })
	end
end

return purifier
