local reminders = {}

function reminders.holiday_eve()
	local perth_offset = 8 * 60 * 60
	local tomorrow = os.date("!%Y-%m-%d", os.time() + perth_offset + 24 * 60 * 60)
	local holiday = holidays.on(tomorrow)

	if holiday == nil then
		return
	end

	notify.send({
		title = holiday,
		message = holiday .. " is tomorrow",
		category = "general",
	})
end

return reminders
