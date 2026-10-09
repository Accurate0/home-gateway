local quiet = gw.lib("purifier")

local purifier = {}

function purifier.enforce_quiet(settings)
	workflow.step("enforce_quiet", function()
		quiet.enforce_quiet(settings.device, event, settings.max_hours)
	end, settings.device)
end

return purifier
