local lights = {}

function lights.all(devices, state)
  gw.require("light:write")

  for _, device in ipairs(devices) do
    light.set(device, { state = state })
  end
end

function lights.any_on(devices)
  gw.require("light:read")

  for _, device in ipairs(devices) do
    if light.is_on(device) then
      return true
    end
  end

  return false
end

return lights
