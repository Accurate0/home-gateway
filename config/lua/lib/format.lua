local format = {}

function format.int(value)
	return string.format("%d", math.floor(value + 0.5))
end

function format.round(value, places)
	local scale = 10 ^ places

	return math.floor(value * scale + 0.5) / scale
end

function format.clock(time)
	local hour, minute = time:match("^(%d+):(%d+)$")

	if hour == nil then
		return time
	end

	hour = tonumber(hour)

	local suffix = hour >= 12 and "pm" or "am"
	local twelve = hour % 12

	if twelve == 0 then
		twelve = 12
	end

	return twelve .. ":" .. minute .. suffix
end

function format.sentences(parts)
	return table.concat(parts, ". ")
end

return format
