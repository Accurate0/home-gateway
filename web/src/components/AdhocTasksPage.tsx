import { useState } from "react";
import { graphql, useLazyLoadQuery, useMutation } from "react-relay";
import { formatDistanceToNow } from "date-fns";
import type { AdhocTasksPageQuery } from "./__generated__/AdhocTasksPageQuery.graphql";
import type { AdhocTasksPageRunCronMutation } from "./__generated__/AdhocTasksPageRunCronMutation.graphql";
import type { AdhocTasksPageRunPendingMutation } from "./__generated__/AdhocTasksPageRunPendingMutation.graphql";
import { cn } from "@/lib/utils";

const AdhocTasksQuery = graphql`
  query AdhocTasksPageQuery {
    adhocCronTasks {
      id
      name
      schedule
      flag
      nextRunAt
      lastRunAt
      durationMs
      rowsAffected
      outcome
    }
    adhocTasks {
      id
      ordinal
      name
      flag
      completedAt
      durationMs
      pending
      checksumDrifted
    }
  }
`;

const RunCronMutation = graphql`
  mutation AdhocTasksPageRunCronMutation($name: String!) {
    runAdhocCronTask(name: $name)
  }
`;

const RunPendingMutation = graphql`
  mutation AdhocTasksPageRunPendingMutation {
    runPendingAdhocTasks
  }
`;

const REFRESH_DELAY_MS = 2000;

const RUN_BUTTON =
  "border-border text-muted-foreground hover:text-foreground cursor-pointer rounded-full border px-3 py-1 text-xs transition-colors disabled:cursor-default disabled:opacity-50";

const OUTCOME_STYLES: Record<string, string> = {
  success: "text-emerald-600 dark:text-emerald-400 border-emerald-500/40",
  error: "text-red-600 dark:text-red-400 border-red-500/40",
  held: "text-amber-600 dark:text-amber-400 border-amber-500/40",
};

function relative(value: string | null | undefined) {
  return value
    ? formatDistanceToNow(new Date(value), { addSuffix: true })
    : "never";
}

export default function AdhocTasksPage() {
  const [fetchKey, setFetchKey] = useState(0);
  const data = useLazyLoadQuery<AdhocTasksPageQuery>(
    AdhocTasksQuery,
    {},
    { fetchKey, fetchPolicy: "store-and-network" },
  );

  const [runningCron, setRunningCron] = useState<string | null>(null);
  const [runningPending, setRunningPending] = useState(false);

  const [commitRunCron] =
    useMutation<AdhocTasksPageRunCronMutation>(RunCronMutation);
  const [commitRunPending] =
    useMutation<AdhocTasksPageRunPendingMutation>(RunPendingMutation);

  const refreshSoon = () =>
    window.setTimeout(() => setFetchKey((key) => key + 1), REFRESH_DELAY_MS);

  const runCron = (name: string) => {
    setRunningCron(name);
    commitRunCron({
      variables: { name },
      onCompleted: () => {
        setRunningCron(null);
        refreshSoon();
      },
      onError: () => setRunningCron(null),
    });
  };

  const runPending = () => {
    setRunningPending(true);
    commitRunPending({
      variables: {},
      onCompleted: () => {
        setRunningPending(false);
        refreshSoon();
      },
      onError: () => setRunningPending(false),
    });
  };

  return (
    <div className="flex flex-col gap-10">
      <section>
        <h2 className="mb-4 text-sm font-medium tracking-wide uppercase">
          Scheduled
        </h2>

        {data.adhocCronTasks.length === 0 ? (
          <p className="text-muted-foreground text-sm">
            No cron tasks registered.
          </p>
        ) : (
          <div className="flex flex-col gap-2">
            {data.adhocCronTasks.map((task) => (
              <div
                key={task.id}
                className="bg-card border-border flex items-center justify-between gap-4 rounded-2xl border p-4"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="truncate font-medium">{task.name}</span>
                    {task.flag && (
                      <span className="text-muted-foreground border-border rounded-full border px-1.5 py-0.5 text-[10px] tracking-wide uppercase">
                        {task.flag}
                      </span>
                    )}
                  </div>
                  <span className="text-muted-foreground font-mono text-xs">
                    {task.schedule}
                    {" · next "}
                    {relative(task.nextRunAt)}
                  </span>
                </div>

                <div className="flex shrink-0 items-center gap-4">
                  <div className="flex flex-col items-end gap-1 text-right">
                    {task.outcome ? (
                      <span
                        className={cn(
                          "rounded-full border px-2 py-0.5 text-[10px] font-medium tracking-wide uppercase",
                          OUTCOME_STYLES[task.outcome] ??
                            "text-muted-foreground border-border",
                        )}
                      >
                        {task.outcome}
                      </span>
                    ) : (
                      <span className="text-muted-foreground border-border rounded-full border px-2 py-0.5 text-[10px] font-medium tracking-wide uppercase">
                        never run
                      </span>
                    )}
                    <span className="text-muted-foreground text-xs">
                      {relative(task.lastRunAt)}
                      {task.durationMs !== null && ` · ${task.durationMs}ms`}
                      {task.rowsAffected !== null &&
                        ` · ${task.rowsAffected} rows`}
                    </span>
                  </div>

                  <button
                    type="button"
                    onClick={() => runCron(task.name)}
                    disabled={runningCron === task.name}
                    className={RUN_BUTTON}
                  >
                    {runningCron === task.name ? "Running…" : "Run now"}
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      <section>
        <div className="mb-4 flex items-center justify-between gap-4">
          <h2 className="text-sm font-medium tracking-wide uppercase">
            One-shot
          </h2>

          <button
            type="button"
            onClick={runPending}
            disabled={runningPending}
            className={RUN_BUTTON}
          >
            {runningPending ? "Running…" : "Run pending"}
          </button>
        </div>

        {data.adhocTasks.length === 0 ? (
          <p className="text-muted-foreground text-sm">
            No one-shot tasks registered.
          </p>
        ) : (
          <div className="flex flex-col gap-2">
            {data.adhocTasks.map((task) => (
              <div
                key={task.id}
                className="bg-card border-border flex items-center justify-between gap-4 rounded-2xl border p-4"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="truncate font-medium">{task.name}</span>
                    {task.checksumDrifted && (
                      <span className="rounded-full border border-red-500/40 px-1.5 py-0.5 text-[10px] tracking-wide text-red-600 uppercase dark:text-red-400">
                        source changed
                      </span>
                    )}
                  </div>
                  <span className="text-muted-foreground font-mono text-xs">
                    {task.ordinal}
                  </span>
                </div>

                <div className="flex shrink-0 flex-col items-end gap-1 text-right">
                  <span
                    className={cn(
                      "rounded-full border px-2 py-0.5 text-[10px] font-medium tracking-wide uppercase",
                      task.pending
                        ? "text-muted-foreground border-border"
                        : OUTCOME_STYLES.success,
                    )}
                  >
                    {task.pending ? "pending" : "applied"}
                  </span>
                  <span className="text-muted-foreground text-xs">
                    {relative(task.completedAt)}
                    {task.durationMs !== null && ` · ${task.durationMs}ms`}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
