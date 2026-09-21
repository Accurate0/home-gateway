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

	entities = {
		"sensor.{name}_status",
		"sensor.{name}_battery",
		"sensor.{name}_current_room",
		"sensor.{name}_vacuum_error",
		"sensor.{name}_dock_dock_error",
		"sensor.{name}_cleaning_progress",
		"sensor.{name}_filter_time_left",
		"sensor.{name}_main_brush_time_left",
		"sensor.{name}_side_brush_time_left",
		"sensor.{name}_sensor_time_left",
		"sensor.{name}_dock_strainer_time_left",
		"binary_sensor.{name}_charging",
		"binary_sensor.{name}_cleaning",
		"binary_sensor.{name}_mop_attached",
		"binary_sensor.{name}_water_shortage",
		"binary_sensor.{name}_water_box_attached",
		"binary_sensor.{name}_dock_clean_water_box",
		"binary_sensor.{name}_dock_dirty_water_box",
		"binary_sensor.{name}_dock_mop_drying",
		"binary_sensor.kitchen_{name}_dock_cleaning_fluid",
	},

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
