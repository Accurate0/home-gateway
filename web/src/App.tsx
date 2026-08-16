import { lazy, Suspense } from "react";
import { NavLink, Navigate, Route, Routes } from "react-router";
import Dashboard from "./components/Dashboard";
import WorkflowsPage from "./components/WorkflowsPage";
import RunsPage from "./components/RunsPage";
import AdhocTasksPage from "./components/AdhocTasksPage";
import HomeAssistantPage from "./components/HomeAssistantPage";
import BatteryPage from "./components/BatteryPage";
import { cn } from "@/lib/utils";

const GraphiqlPage = lazy(() => import("./components/GraphiqlPage"));

const TABS = [
  { to: "/dashboard", label: "Dashboard" },
  { to: "/workflows", label: "Workflows" },
  { to: "/runs", label: "Runs" },
  { to: "/adhoc-tasks", label: "Adhoc Tasks" },
  { to: "/batteries", label: "Batteries" },
  { to: "/home-assistant", label: "Home Assistant" },
  { to: "/graphql", label: "GraphQL" },
];

function Fallback() {
  return <p className="text-muted-foreground text-sm">Loading…</p>;
}

export default function App() {
  return (
    <div className="mx-auto max-w-6xl px-6 py-12">
      <header className="mb-8">
        <h1 className="font-display text-4xl font-semibold tracking-tight sm:text-5xl">
          Home Gateway
        </h1>
      </header>

      <nav className="border-border mb-8 flex gap-1 border-b">
        {TABS.map((t) => (
          <NavLink
            key={t.to}
            to={t.to}
            className={({ isActive }) =>
              cn(
                "-mb-px cursor-pointer border-b-2 border-transparent px-3 pb-2 text-sm font-medium transition-colors",
                isActive
                  ? "text-foreground border-foreground"
                  : "text-muted-foreground hover:text-foreground",
              )
            }
          >
            {t.label}
          </NavLink>
        ))}
      </nav>

      <Suspense fallback={<Fallback />}>
        <Routes>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/workflows" element={<WorkflowsPage />} />
          <Route path="/runs" element={<RunsPage />} />
          <Route path="/adhoc-tasks" element={<AdhocTasksPage />} />
          <Route path="/batteries" element={<BatteryPage />} />
          <Route path="/home-assistant" element={<HomeAssistantPage />} />
          <Route path="/graphql" element={<GraphiqlPage />} />
        </Routes>
      </Suspense>
    </div>
  );
}
