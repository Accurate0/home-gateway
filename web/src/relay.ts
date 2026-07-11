import {
  Environment,
  Network,
  Observable,
  RecordSource,
  Store,
  type FetchFunction,
  type SubscribeFunction,
} from "relay-runtime";
import { createClient } from "graphql-ws";
import { getAccessToken } from "./auth";

// Same-origin relative endpoints: in production the SPA and the API share the
// `home.inf-k8s.net` host; in dev Vite proxies `/v1` to the local backend.
const HTTP_ENDPOINT = "/v1/graphql";

function wsUrl(): string {
  const proto = window.location.protocol === "https:" ? "wss" : "ws";
  return `${proto}://${window.location.host}/v1/graphql/ws`;
}

function authHeaders(): Record<string, string> {
  const token = getAccessToken();
  return token ? { Authorization: `Bearer ${token}` } : {};
}

const fetchFn: FetchFunction = async (request, variables) => {
  const resp = await fetch(HTTP_ENDPOINT, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...authHeaders(),
    },
    body: JSON.stringify({ query: request.text, variables }),
  });
  if (!resp.ok) {
    throw new Error(`GraphQL request failed: ${resp.status}`);
  }
  return await resp.json();
};

// The backend reads the token from the graphql-ws connection_init payload
// (see token_from_payload in src/graphql/handler.rs).
const wsClient = createClient({
  url: wsUrl(),
  connectionParams: () => authHeaders(),
});

const subscribeFn: SubscribeFunction = (operation, variables) => {
  return Observable.create((sink) => {
    return wsClient.subscribe(
      {
        operationName: operation.name,
        query: operation.text ?? "",
        variables,
      },
      {
        next: (value) => sink.next(value as never),
        error: (err) => sink.error(err as Error),
        complete: () => sink.complete(),
      },
    );
  });
};

export const environment = new Environment({
  network: Network.create(fetchFn, subscribeFn),
  store: new Store(new RecordSource()),
  getDataID: (fieldValue, typeName) =>
    `${typeName}:${(fieldValue as { id?: string }).id ?? ""}`,
});
