---@meta

---@alias LuaTestSuite table<string, fun()>

---@class TestSpy
---@field calls any[][] the arguments of every call, in order
---@overload fun(...): any

---@class test
test = {}

---Loads `config/lua/<path>.lua`; `models/` modules see only the model library as `gw.lib`
---@param path string
---@return any
function test.load(path) end

---Fails unless `actual` deeply equals `expected`
---@param actual any
---@param expected any
---@param message string?
function test.eq(actual, expected, message) end

---Fails unless `run` raises an error containing `substring`; returns the message
---@param run fun()
---@param substring string
---@return string
function test.errors(run, substring) end

---A callable that records its arguments in `calls` and returns `impl(...)`
---@param impl fun(...): any?
---@return TestSpy
function test.spy(impl) end

---@param value any
---@return string
function test.inspect(value) end
