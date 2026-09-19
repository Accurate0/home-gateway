import { Suspense, useEffect, useMemo, useState } from "react";
import { graphql, useLazyLoadQuery } from "react-relay";
import { useSearchParams } from "react-router";
import { formatDistanceToNow } from "date-fns";
import { ChevronRight } from "lucide-react";
import type { RunsPageQuery } from "./__generated__/RunsPageQuery.graphql";
import RunTrace from "./RunTrace";
import { cn } from "@/lib/utils";

const RunsQuery = graphql`
  query RunsPageQuery {
    workflowRuns(limit: 100) {
      id
      eventId
      slug
      name
      outcome
      dryRun
      durationMs
      error
      startedAt
    }
  }
`;

const OUTCOME_STYLES: Record<string, string> = {
  success: "text-emerald-600 dark:text-emerald-400 border-emerald-500/40",
  error: "text-red-600 dark:text-red-400 border-red-500/40",
  disabled: "text-muted-foreground border-border",
};

const PENDING_POLL_MS = 1500;
const PENDING_POLL_LIMIT = 10;

export default function RunsPage() {
  const [searchParams] = useSearchParams();
  const pendingEventId = searchParams.get("run");
  const [fetchKey, setFetchKey] = useState(0);
  const data = useLazyLoadQuery<RunsPageQuery>(
    RunsQuery,
    {},
    { fetchKey, fetchPolicy: "store-and-network" },
  );
  const [filter, setFilter] = useState<string | null>(null);
  const [toggled, setToggled] = useState<Set<string>>(new Set());

  const pendingRun = pendingEventId
    ? data.workflowRuns.find((r) => r.eventId === pendingEventId)
    : undefined;

  useEffect(() => {
    if (!pendingEventId || pendingRun || fetchKey >= PENDING_POLL_LIMIT) return;

    const timer = setTimeout(() => setFetchKey((k) => k + 1), PENDING_POLL_MS);

    return () => clearTimeout(timer);
  }, [pendingEventId, pendingRun, fetchKey]);

  const isExpanded = (id: string) =>
    toggled.has(id) !== (id === pendingRun?.id);

  const toggle = (id: string) =>
    setToggled((prev) => {
      const next = new Set(prev);

      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }

      return next;
    });

  const runs = useMemo(
    () =>
      filter
        ? data.workflowRuns.filter((r) => r.slug === filter)
        : data.workflowRuns,
    [data.workflowRuns, filter],
  );

  const slugs = useMemo(
    () => [...new Set(data.workflowRuns.map((r) => r.slug))].sort(),
    [data.workflowRuns],
  );

  return (
    <div>
      <div className="mb-6 flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={() => setFilter(null)}
          className={cn(
            "rounded-full border px-3 py-1 text-xs transition-colors",
            filter === null
              ? "border-foreground text-foreground"
              : "border-border text-muted-foreground hover:text-foreground",
          )}
        >
          All
        </button>
        {slugs.map((slug) => (
          <button
            key={slug}
            type="button"
            onClick={() => setFilter(slug)}
            className={cn(
              "rounded-full border px-3 py-1 font-mono text-xs transition-colors",
              filter === slug
                ? "border-foreground text-foreground"
                : "border-border text-muted-foreground hover:text-foreground",
            )}
          >
            {slug}
          </button>
        ))}
      </div>

      {runs.length === 0 ? (
        <p className="text-muted-foreground text-sm">
          No workflow runs recorded yet.
        </p>
      ) : (
        <div className="flex flex-col gap-2">
          {runs.map((run) => (
            <div
              key={run.id}
              className={cn(
                "bg-card border-border rounded-2xl border p-4",
                run.eventId === pendingEventId && "border-sky-500/60",
              )}
            >
              <button
                type="button"
                onClick={() => toggle(run.id)}
                aria-expanded={isExpanded(run.id)}
                className="flex w-full cursor-pointer items-center justify-between gap-4 text-left"
              >
                <div className="flex min-w-0 items-start gap-2">
                  <ChevronRight
                    className={cn(
                      "text-muted-foreground mt-1 size-4 shrink-0 transition-transform",
                      isExpanded(run.id) && "rotate-90",
                    )}
                    aria-hidden
                  />
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="truncate font-medium">{run.name}</span>
                      {run.dryRun && (
                        <span className="text-muted-foreground border-border rounded-full border px-1.5 py-0.5 text-[10px] tracking-wide uppercase">
                          dry run
                        </span>
                      )}
                    </div>
                    <span className="text-muted-foreground font-mono text-xs">
                      {run.slug}
                    </span>
                    {run.error && (
                      <p className="mt-1 truncate font-mono text-xs text-red-600 dark:text-red-400">
                        {run.error}
                      </p>
                    )}
                  </div>
                </div>
                <div className="flex shrink-0 flex-col items-end gap-1 text-right">
                  <span
                    className={cn(
                      "rounded-full border px-2 py-0.5 text-[10px] font-medium tracking-wide uppercase",
                      OUTCOME_STYLES[run.outcome] ??
                        "text-muted-foreground border-border",
                    )}
                  >
                    {run.outcome}
                  </span>
                  <span className="text-muted-foreground text-xs">
                    {formatDistanceToNow(new Date(run.startedAt), {
                      addSuffix: true,
                    })}
                    {" · "}
                    {run.durationMs}ms
                  </span>
                </div>
              </button>
              {isExpanded(run.id) && (
                <Suspense
                  fallback={
                    <p className="text-muted-foreground mt-4 text-xs">
                      Loading trace…
                    </p>
                  }
                >
                  <RunTrace runId={run.id} eventId={run.eventId} />
                </Suspense>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
