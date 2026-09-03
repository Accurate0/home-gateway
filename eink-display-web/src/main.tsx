import { Component, StrictMode, Suspense, type ReactNode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import StatusPanel from "./components/StatusPanel.tsx";
import { formatUpdatedAt } from "./lib/time.ts";
import { RelayEnvironmentProvider } from "react-relay";
import { Environment, Network, type FetchFunction } from "relay-runtime";

const HTTP_ENDPOINT = import.meta.env.DEV
  ? "http://localhost:8000/v1/graphql"
  : "https://home.anurag.sh/v1/graphql";

const fetchGraphQL: FetchFunction = async (request, variables) => {
  const resp = await fetch(HTTP_ENDPOINT, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Api-Key": import.meta.env.VITE_GRAPHQL_API_KEY,
    },
    body: JSON.stringify({ query: request.text, variables }),
  });
  if (!resp.ok) {
    throw new Error("Response failed.");
  }
  return await resp.json();
};

const environment = new Environment({
  network: Network.create(fetchGraphQL),
});

class PanelErrorBoundary extends Component<
  { children: ReactNode },
  { failed: boolean }
> {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  render() {
    if (this.state.failed) {
      return (
        <StatusPanel
          message="Data unavailable"
          updatedAt={formatUpdatedAt(new Date())}
        />
      );
    }

    return this.props.children;
  }
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RelayEnvironmentProvider environment={environment}>
      <PanelErrorBoundary>
        <Suspense
          fallback={
            <StatusPanel
              message="Loading"
              updatedAt={formatUpdatedAt(new Date())}
            />
          }
        >
          <App />
        </Suspense>
      </PanelErrorBoundary>
    </RelayEnvironmentProvider>
  </StrictMode>,
);
