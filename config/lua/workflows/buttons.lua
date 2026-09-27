local buttons = {}

function buttons.dispatch(device, bindings)
	local command = bindings[event.action]

	if command == nil then
		gw.log("no binding for `" .. tostring(event.action) .. "` on " .. device)
		return
	end

	workflow.step("binding", function()
		light.set(device, command)
	end, tostring(event.action))
end

return buttons
