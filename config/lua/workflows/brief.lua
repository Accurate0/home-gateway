local format = gw.lib("format")

local brief = {}

local function message(parts)
	return { message = format.sentences(parts) }
end

function brief.morning()
	local today = willyweather.today
	local parts = {}

	local holiday = holidays.on()

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

		if today.rain_probability >= 60 then
			table.insert(parts, "Take an umbrella")
		end
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
