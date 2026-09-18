---@type HomeAssistantModel
return {
	roles = { "media_player" },

	decode = function(entity)
		return {
			media_player = {
				state = entity.state,
				attributes = entity.attributes,
			},
		}
	end,
}
