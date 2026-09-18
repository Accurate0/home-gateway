local UNREPORTED = { unavailable = true, unknown = true }

local FIELDS = {
	status = "_status$",
	room = "_current_room$",
	battery = "_battery$",
}

local function object_id(entity_id)
	return entity_id:match("^[^.]+%.(.+)$") or entity_id
end

local function value(entity)
	if entity.entity_id:match("^binary_sensor%.") then
		return entity.state == "on"
	end

	return tonumber(entity.state) or entity.state
end

---@type HomeAssistantModel
return {
	roles = { "robot_vacuum", "battery" },

	decode = function(entity)
		if UNREPORTED[entity.state] then
			return {}
		end

		if entity.entity_id:match("^sensor%.") then
			for field, pattern in pairs(FIELDS) do
				if entity.entity_id:match(pattern) then
					local reported = field == "battery" and tonumber(entity.state) or entity.state

					return { robot_vacuum = { [field] = reported } }
				end
			end
		end

		return { metrics = { [object_id(entity.entity_id)] = value(entity) } }
	end,
}
