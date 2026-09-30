local synergy = test.load("integrations/synergy")

local HEADER = "Date,Time,ANYTIME (KWH),Solar export (Units)"

local function stub_time()
	time = {
		parse = function(iso)
			return iso
		end,
	}
end

---@type LuaTestSuite
return {
	["parses each line into an interval"] = function()
		stub_time()

		local intervals = synergy.parse(HEADER .. "\r\n30/09/2026,07:30,0.25,0.5\r\n30/09/2026,07:45,0.1,0\r\n")

		test.eq(intervals, {
			{ used = 0.25, exported = 0.5, at = "2026-09-30T07:30:00+08:00" },
			{ used = 0.1, exported = 0, at = "2026-09-30T07:45:00+08:00" },
		})
	end,

	["columns are found by name, not position"] = function()
		stub_time()

		local intervals = synergy.parse("Solar export (Units),ANYTIME (KWH),Time,Date\n1.5,2,23:00,1/10/2026")

		test.eq(intervals, { { used = 2, exported = 1.5, at = "2026-10-01T23:00:00+08:00" } })
	end,

	["a header without the energy columns is rejected"] = function()
		test.errors(function()
			synergy.parse("Date,Time\n30/09/2026,07:30")
		end, "synergy export is missing `ANYTIME (KWH)`, `Solar export (Units)`")
	end,

	["a malformed line names its line number"] = function()
		stub_time()

		test.errors(function()
			synergy.parse(HEADER .. "\n30/09/2026,07:30,0.25,0.5\nnot,an,interval,row")
		end, "synergy export line 3 is not an interval")
	end,
}
