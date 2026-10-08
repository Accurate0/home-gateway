local quiet = gw.lib("purifier")

local purifier = {}

function purifier.enforce(settings)
	workflow.step("enforce", function()
		quiet.enforce(settings.device, event, settings.max_hours)
	end, settings.device)
end

return purifier
