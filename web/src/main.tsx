import { StrictMode, Suspense } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import { RelayEnvironmentProvider } from "react-relay";
import { environment } from "./relay";
import { ensureAuthenticated } from "./auth";

await ensureAuthenticated();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RelayEnvironmentProvider environment={environment}>
      <Suspense
        fallback={
          <div className="text-muted-foreground grid min-h-screen place-items-center">
            Loading…
          </div>
        }
      >
        <App />
      </Suspense>
    </RelayEnvironmentProvider>
  </StrictMode>,
);
