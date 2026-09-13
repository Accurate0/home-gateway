local lights = gw.lib("lights")

local buttons = {}

function buttons.dispatch(device, bindings)
	local command = bindings[event.action]

	if command == nil then
		gw.log("no binding for `" .. tostring(event.action) .. "` on " .. device)
		return
	end

	light.set(device, command)
end

function buttons.toggle(device, on_workflow, off_workflow)
	if lights.any_on({ device }) then
		workflow.run(off_workflow)
	else
		workflow.run(on_workflow)
	end
end

return buttons
