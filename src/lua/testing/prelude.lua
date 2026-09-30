test = {}

local function deep_equal(a, b)
	if a == b then
		return true
	end

	if type(a) ~= "table" or type(b) ~= "table" then
		return false
	end

	for key, value in pairs(a) do
		if not deep_equal(value, b[key]) then
			return false
		end
	end

	for key in pairs(b) do
		if a[key] == nil then
			return false
		end
	end

	return true
end

function test.eq(actual, expected, message)
	if deep_equal(actual, expected) then
		return
	end

	local prefix = message and (message .. "\n") or ""

	error(prefix .. "expected: " .. test.inspect(expected) .. "\n  actual: " .. test.inspect(actual), 2)
end

function test.errors(run, substring)
	local ok, err = pcall(run)

	if ok then
		error("expected an error containing `" .. substring .. "`, but none was raised", 2)
	end

	local message = tostring(err)

	if not message:find(substring, 1, true) then
		error("expected an error containing `" .. substring .. "`, got: " .. message, 2)
	end

	return message
end

function test.spy(impl)
	local spy = { calls = {} }

	return setmetatable(spy, {
		__call = function(_, ...)
			table.insert(spy.calls, { ... })

			if impl ~= nil then
				return impl(...)
			end
		end,
	})
end

gw = {
	event_id = "test",
	origin = "test",
	dry_run = false,
}

function gw.sleep() end

function gw.defer(_, _, run)
	run()
end

function gw.has()
	return true
end

function gw.require() end

for _, name in ipairs({ "http", "graphql", "cooldown", "emit" }) do
	gw[name] = function()
		error("gw." .. name .. " is not stubbed; assign gw." .. name .. " in the test", 2)
	end
end
