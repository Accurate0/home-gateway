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

return reminders
