export type LuaExample = {
  name: string;
  description: string;
  script: string;
};

export const LUA_EXAMPLES: readonly LuaExample[] = [
  {
    name: "Hello",
    description: "Log a line and return the call context",
    script: `gw.log("hello from the playground")

return {
  origin = gw.origin,
  event_id = gw.event_id,
  dry_run = gw.dry_run,
}
`,
  },
  {
    name: "Devices offline",
    description: "List devices unseen for an hour",
    script: `local stale = device.offline(60)

for _, id in ipairs(stale) do
  gw.log("offline: " .. id)
end

return stale
`,
  },
  {
    name: "Shared state",
    description: "Read, bump and clear a counter",
    script: `local key = "playground.counter"

state.incr(key, 1)
local value = state.get(key)
state.clear(key)

return value
`,
  },
  {
    name: "Notify",
    description: "Send a push notification (respects dry run)",
    script: `notify.send({
  title = "Playground",
  message = "Sent from the Lua playground",
  category = "general",
})

return "sent"
`,
  },
  {
    name: "HTTP + JSON",
    description: "Call an external service and decode the body",
    script: `local response = gw.http({
  url = "https://httpbin.org/json",
  method = "GET",
})

if not response.ok then
  error("request failed with " .. response.status)
end

return json.decode(response.body)
`,
  },
  {
    name: "GraphQL",
    description: "Query the gateway's own API (queries only)",
    script: `local result = gw.graphql([[
  query {
    mode { active }
  }
]], {})

return result
`,
  },
];
