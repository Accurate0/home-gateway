import { StrictMode, Suspense } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import { RelayEnvironmentProvider } from "react-relay";
import { environment } from "./relay";
import { ensureAuthenticated } from "./auth";

const root = createRoot(document.getElementById("root")!);

try {
  await ensureAuthenticated();
  root.render(
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
} catch (e) {
  const message = e instanceof Error ? e.message : String(e);
  root.render(
    <div className="mx-auto grid min-h-screen max-w-md place-items-center px-6 text-center">
      <div>
        <h1 className="mb-2 text-lg font-semibold">Sign-in failed</h1>
        <p className="text-muted-foreground mb-4 text-sm">{message}</p>
        <a href="/" className="text-sm underline">
          Try again
        </a>
      </div>
    </div>,
  );
}
