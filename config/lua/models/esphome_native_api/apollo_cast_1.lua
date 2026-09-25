---@type EsphomeNativeApiModel
return {
	roles = { "light", "media_player" },
	watchdog = "1h",
	capabilities = { "brightness", "rgb" },
	ranges = { brightness = { min = 0, max = 255 } },

	encode = { light = gw.lib("esphome_light") },

	decode = function(input)
		if input.domain == "light" then
			return { light = input.payload }
		end

		if input.domain == "media_player" then
			local entities = input.entities

			return {
				media_player = {
					state = input.payload.state,
					attributes = {
						media_title = entities.track_title,
						media_artist = entities.artist,
						media_album_name = entities.album,
						volume_level = input.payload.volume,
						is_volume_muted = input.payload.muted,
						app_name = "Sendspin",
					},
				},
			}
		end

		return { metrics = { [input.object_id] = input.payload } }
	end,
}
