import { useMemo, useState } from "react";
import { graphql, useLazyLoadQuery } from "react-relay";
import { formatDistanceToNow } from "date-fns";
import type { RunsPageQuery } from "./__generated__/RunsPageQuery.graphql";
import { cn } from "@/lib/utils";

const RunsQuery = graphql`
  query RunsPageQuery {
    workflowRuns(limit: 100) {
      id
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

export default function RunsPage() {
  const data = useLazyLoadQuery<RunsPageQuery>(RunsQuery, {});
  const [filter, setFilter] = useState<string | null>(null);

  const runs = useMemo(
    () =>
      filter ? data.workflowRuns.filter((r) => r.slug === filter) : data.workflowRuns,
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
        <p className="text-muted-foreground text-sm">No workflow runs recorded yet.</p>
      ) : (
        <div className="flex flex-col gap-2">
          {runs.map((run) => (
            <div
              key={run.id}
              className="bg-card border-border flex items-center justify-between gap-4 rounded-2xl border p-4"
            >
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
              <div className="flex shrink-0 flex-col items-end gap-1 text-right">
                <span
                  className={cn(
                    "rounded-full border px-2 py-0.5 text-[10px] font-medium tracking-wide uppercase",
                    OUTCOME_STYLES[run.outcome] ?? "text-muted-foreground border-border",
                  )}
                >
                  {run.outcome}
                </span>
                <span className="text-muted-foreground text-xs">
                  {formatDistanceToNow(new Date(run.startedAt), { addSuffix: true })}
                  {" · "}
                  {run.durationMs}ms
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
