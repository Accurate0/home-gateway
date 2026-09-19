import { graphql, useLazyLoadQuery } from "react-relay";
import type { RunTraceQuery } from "./__generated__/RunTraceQuery.graphql";
import { cn } from "@/lib/utils";

const TraceQuery = graphql`
  query RunTraceQuery($eventId: UUID!) {
    workflowRuns(eventId: $eventId) {
      id
      trigger
      steps {
        seq
        depth
        kind
        outcome
        guard
        detail
        error
        durationMs
      }
    }
  }
`;

const STEP_DOT: Record<string, string> = {
  ran: "bg-emerald-500",
  dry_run: "bg-sky-500",
  guard_skipped: "bg-muted-foreground/40",
  error: "bg-red-500",
  running: "bg-amber-500",
};

const STEP_LABEL: Record<string, string> = {
  ran: "ran",
  dry_run: "dry run",
  guard_skipped: "skipped",
  error: "error",
  running: "unfinished",
};

export default function RunTrace({
  runId,
  eventId,
}: {
  runId: string;
  eventId: string;
}) {
  const data = useLazyLoadQuery<RunTraceQuery>(
    TraceQuery,
    { eventId },
    { fetchPolicy: "store-or-network" },
  );

  const run = data.workflowRuns.find((r) => r.id === runId);

  if (!run) {
    return <p className="text-muted-foreground text-xs">Trace not found.</p>;
  }

  return (
    <div className="border-border mt-4 border-t pt-4">
      {run.steps.length === 0 ? (
        <p className="text-muted-foreground text-xs">No steps recorded.</p>
      ) : (
        <ol className="flex flex-col">
          {run.steps.map((step) => (
            <li
              key={step.seq}
              className="relative flex gap-3 py-1.5"
              style={{ paddingLeft: `${step.depth * 1.25}rem` }}
            >
              <span
                className={cn(
                  "mt-1.5 size-2 shrink-0 rounded-full",
                  STEP_DOT[step.outcome] ?? "bg-muted-foreground",
                )}
                aria-hidden
              />
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-baseline gap-x-2">
                  <span
                    className={cn(
                      "font-mono text-xs",
                      step.outcome === "guard_skipped" &&
                        "text-muted-foreground line-through",
                    )}
                  >
                    {step.kind}
                  </span>
                  <span className="text-muted-foreground text-[10px] tracking-wide uppercase">
                    {STEP_LABEL[step.outcome] ?? step.outcome}
                  </span>
                  {step.durationMs > 0 && (
                    <span className="text-muted-foreground text-[10px]">
                      {step.durationMs}ms
                    </span>
                  )}
                </div>
                {step.detail && (
                  <p className="text-muted-foreground truncate font-mono text-xs">
                    {step.detail}
                  </p>
                )}
                {step.guard && (
                  <p className="text-muted-foreground font-mono text-xs">
                    when {step.guard}
                  </p>
                )}
                {step.error && (
                  <p className="font-mono text-xs break-words text-red-600 dark:text-red-400">
                    {step.error}
                  </p>
                )}
              </div>
            </li>
          ))}
        </ol>
      )}

      {run.trigger != null && (
        <details className="mt-3">
          <summary className="text-muted-foreground cursor-pointer text-xs">
            Trigger variables
          </summary>
          <pre className="bg-muted mt-2 overflow-x-auto rounded-lg p-3 font-mono text-xs">
            {JSON.stringify(run.trigger, null, 2)}
          </pre>
        </details>
      )}
    </div>
  );
}
