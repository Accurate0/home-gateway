local format = test.load("lib/format")

---@type LuaTestSuite
return {
	["int rounds half up"] = function()
		test.eq(format.int(2.5), "3")
		test.eq(format.int(2.4), "2")
	end,

	["round keeps the requested places"] = function()
		test.eq(format.round(1.2345, 2), 1.23)
		test.eq(format.round(18.96, 0), 19)
	end,

	["clock renders a 12 hour time"] = function()
		test.eq(format.clock("07:05"), "7:05am")
		test.eq(format.clock("13:30"), "1:30pm")
		test.eq(format.clock("00:15"), "12:15am")
		test.eq(format.clock("12:00"), "12:00pm")
	end,

	["clock reads the time out of an iso timestamp"] = function()
		test.eq(format.clock("2026-09-30T18:45:00+08:00"), "6:45pm")
	end,

	["clock passes through anything it cannot parse"] = function()
		test.eq(format.clock("soon"), "soon")
	end,

	["sentences joins parts with full stops"] = function()
		test.eq(format.sentences({ "Sunny", "Rain 10%" }), "Sunny. Rain 10%")
	end,
}
