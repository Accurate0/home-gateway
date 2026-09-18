local format = gw.lib("format")

local prices = {}

function prices.cheapest_fuel()
	local sites = fuel.cheapest(nil, 1)
	local site = sites and sites[1]

	if site == nil then
		return { message = "No fuel prices available today" }
	end

	local message = string.format(
		"Cheapest ULP91 near %s: %sc/L at %s %s",
		site.suburb,
		format.round(site.price, 1),
		site.brand,
		site.name
	)

	return { message = message }
end

return prices
