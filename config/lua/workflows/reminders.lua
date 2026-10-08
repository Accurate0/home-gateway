local reminders = {}

function reminders.holiday_eve()
	local perth_offset = 8 * 60 * 60
	local tomorrow = os.date("!%Y-%m-%d", os.time() + perth_offset + 24 * 60 * 60)

	local holiday = workflow.step("check", function()
		return holidays.on(tomorrow)
	end, tomorrow)

	if holiday == nil then
		return
	end

	workflow.step("notify", function()
		notify.send({
			title = holiday,
			message = holiday .. " is tomorrow",
			category = "general",
		})
	end, holiday)
end

function reminders.filter_low(settings)
	local filter_life = workflow.step("check", function()
		local state = air_purifier.state(settings.device)

		return state and state.filter_life
	end, settings.device)

	if filter_life == nil then
		gw.log("no filter life reported for " .. settings.device)
		return
	end

	if filter_life >= settings.below then
		return
	end

	local remaining = math.floor(filter_life + 0.5)

	workflow.step("notify", function()
		notify.send({
			title = settings.name,
			message = settings.name .. " filter is at " .. remaining .. "%, time to replace it",
			category = "general",
			tag = "filter_low:" .. settings.device,
		})
	end, settings.device)
end

return reminders
