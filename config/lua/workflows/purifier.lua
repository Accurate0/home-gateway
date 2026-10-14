local purifier = {}

function purifier.auto(settings)
	workflow.step("purifier_auto", function()
		local current = air_purifier.state(settings.device)

		if current ~= nil and current.on == true and current.mode == "auto" then
			gw.log(settings.device .. " is already on auto")
			return
		end

		air_purifier.command(settings.device, { command = "set_mode", mode = "auto" })
	end, settings.device)
end

return purifier
