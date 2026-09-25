local synergy = {}

local columns = {
	date = "Date",
	time = "Time",
	used = "ANYTIME (KWH)",
	exported = "Solar export (Units)",
}

local offset = "+08:00"

local function split(line)
	local fields = {}

	for field in (line .. ","):gmatch("([^,]*),") do
		table.insert(fields, field)
	end

	return fields
end

local function positions(header)
	local found = {}

	for index, name in ipairs(split(header)) do
		found[name] = index
	end

	local result = {}

	for key, name in pairs(columns) do
		if found[name] == nil then
			error("synergy export has no `" .. name .. "` column, header is: " .. header)
		end

		result[key] = found[name]
	end

	return result
end

local function interval(fields, at, number)
	local day, month, year = fields[at.date]:match("^(%d+)/(%d+)/(%d+)$")
	local hour, minute = fields[at.time]:match("^(%d+):(%d+)$")
	local used = tonumber(fields[at.used])
	local exported = tonumber(fields[at.exported])

	if day == nil or hour == nil or used == nil or exported == nil then
		error("synergy export line " .. number .. " is not an interval: " .. table.concat(fields, ","))
	end

	local iso = string.format("%04d-%02d-%02dT%02d:%02d:00%s", year, month, day, hour, minute, offset)

	return { used = used, exported = exported, at = time.parse(iso) }
end

function synergy.parse(csv)
	local intervals = {}
	local at = nil
	local number = 0

	for line in csv:gmatch("[^\r\n]+") do
		number = number + 1

		if at == nil then
			at = positions(line)
		else
			table.insert(intervals, interval(split(line), at, number))
		end
	end

	return intervals
end

return synergy
