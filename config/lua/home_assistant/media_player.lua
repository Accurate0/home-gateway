---@type HomeAssistantModel
return {
	roles = { "media_player" },
	watchdog = "12h",

	decode = function(entity)
		return {
			media_player = {
				state = entity.state,
				attributes = entity.attributes,
			},
		}
	end,
}
