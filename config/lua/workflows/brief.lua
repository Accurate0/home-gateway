local format = gw.lib("format")

local brief = {}

local function message(parts)
	return { message = format.sentences(parts) }
end

function brief.morning()
	local today, holiday = workflow.step("fetch", function()
		return weather.today(), holidays.on()
	end, "forecast and holiday")

	if today == nil then
		return { message = "No forecast available this morning" }
	end

	return workflow.step("compose", function()
		return brief.compose(today, holiday)
	end)
end

function brief.compose(today, holiday)
	local parts = {}

	if holiday then
		table.insert(parts, holiday)
	end

	table.insert(parts, string.format("%s %s, %s-%s°C", today.emoji, today.description, today.min, today.max))

	if today.rain_probability then
		local rain = "Rain " .. format.int(today.rain_probability) .. "%"

		if today.rain_range then
			rain = rain .. " (" .. today.rain_range .. "mm)"
		end

		table.insert(parts, rain)
	end

	if today.wind_max_speed then
		table.insert(parts, "Wind up to " .. format.int(today.wind_max_speed) .. "km/h")
	end

	if today.uv then
		table.insert(parts, "UV " .. format.round(today.uv, 1))
	end

	if today.sunset then
		table.insert(parts, "Sunset " .. format.clock(today.sunset))
	end

	return message(parts)
end

return brief
