local lights = gw.lib("lights")

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

function buttons.toggle(device, on_workflow, off_workflow)
	local on = workflow.step("check", function()
		return lights.any_on({ device })
	end, device)

	local target = on and off_workflow or on_workflow

	workflow.step("toggle", function()
		workflow.run(target)
	end, target)
end

return buttons
