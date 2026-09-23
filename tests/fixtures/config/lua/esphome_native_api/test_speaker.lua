---@type EsphomeNativeApiModel
return {
	roles = { "media_player" },
	watchdog = "1h",

	decode = function(input)
		if input.domain == "media_player" then
			return {
				media_player = {
					state = input.payload.state,
					attributes = { media_title = input.entities.track_title },
				},
			}
		end

		return { metrics = { [input.object_id] = input.payload } }
	end,
}
