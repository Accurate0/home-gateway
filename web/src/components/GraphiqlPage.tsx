import { useMemo } from "react";
import { GraphiQL } from "graphiql";
import { createGraphiQLFetcher } from "@graphiql/toolkit";
import "graphiql/setup-workers/vite";
import "graphiql/style.css";
import { AUTH_DISABLED, getAccessToken, getApiKey } from "../auth";

function authHeaders(): Record<string, string> {
  if (AUTH_DISABLED) {
    const key = getApiKey();
    return key ? { "X-Api-Key": key } : {};
  }
  const token = getAccessToken();
  return token ? { Authorization: `Bearer ${token}` } : {};
}

function wsUrl(): string {
  const proto = window.location.protocol === "https:" ? "wss" : "ws";
  return `${proto}://${window.location.host}/v1/graphql/ws`;
}

export default function GraphiqlPage() {
  const fetcher = useMemo(
    () =>
      createGraphiQLFetcher({
        url: "/v1/graphql",
        subscriptionUrl: wsUrl(),
        fetch: (input, init) =>
          fetch(input as RequestInfo, {
            ...init,
            headers: { ...init?.headers, ...authHeaders() },
          }),
        wsConnectionParams: authHeaders,
      }),
    [],
  );

  return (
    <div className="graphiql-container h-[calc(100vh-11rem)] min-h-[600px] w-full overflow-hidden rounded-xl">
      <GraphiQL fetcher={fetcher} />
    </div>
  );
}
