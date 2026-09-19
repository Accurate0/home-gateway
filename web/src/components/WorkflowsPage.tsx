import { useMemo, useState } from "react";
import { graphql, useLazyLoadQuery, useMutation } from "react-relay";
import type { WorkflowsPageQuery } from "./__generated__/WorkflowsPageQuery.graphql";
import { useNavigate } from "react-router";
import type { WorkflowsPageSetEnabledMutation } from "./__generated__/WorkflowsPageSetEnabledMutation.graphql";
import type { WorkflowsPageDryRunMutation } from "./__generated__/WorkflowsPageDryRunMutation.graphql";
import { Switch } from "./ui/switch";

const WorkflowsQuery = graphql`
  query WorkflowsPageQuery {
    workflows {
      id
      slug
      name
      group
      enabled
      configEnabled
      dryRun
      reusable
      modes
    }
  }
`;

const SetEnabledMutation = graphql`
  mutation WorkflowsPageSetEnabledMutation($slug: String!, $enabled: Boolean!) {
    setWorkflowEnabled(slug: $slug, enabled: $enabled)
  }
`;

const DryRunMutation = graphql`
  mutation WorkflowsPageDryRunMutation($slug: String!) {
    runWorkflow(slug: $slug, dryRun: true)
  }
`;

type Workflow = WorkflowsPageQuery["response"]["workflows"][number];

export default function WorkflowsPage() {
  const data = useLazyLoadQuery<WorkflowsPageQuery>(WorkflowsQuery, {});
  const [overrides, setOverrides] = useState<Map<string, boolean>>(new Map());
  const [commit] =
    useMutation<WorkflowsPageSetEnabledMutation>(SetEnabledMutation);
  const [commitDryRun, dryRunInFlight] =
    useMutation<WorkflowsPageDryRunMutation>(DryRunMutation);
  const [dryRunError, setDryRunError] = useState<string | null>(null);
  const navigate = useNavigate();

  const dryRun = (w: Workflow) => {
    setDryRunError(null);
    commitDryRun({
      variables: { slug: w.slug },
      onCompleted: (response) => {
        void navigate(`/runs?run=${encodeURIComponent(response.runWorkflow)}`);
      },
      onError: (error) => setDryRunError(`${w.name}: ${error.message}`),
    });
  };

  const workflows = useMemo(
    () =>
      data.workflows.map((w) => ({
        ...w,
        enabled: overrides.get(w.slug) ?? w.enabled,
      })),
    [data.workflows, overrides],
  );

  const groups = useMemo(() => {
    const byGroup = new Map<string, typeof workflows>();
    for (const w of workflows) {
      const list = byGroup.get(w.group) ?? [];
      list.push(w);
      byGroup.set(w.group, list);
    }
    return [...byGroup.entries()].sort(([a], [b]) => a.localeCompare(b));
  }, [workflows]);

  const setLocal = (slug: string, enabled: boolean) =>
    setOverrides((prev) => new Map(prev).set(slug, enabled));

  const toggle = (w: Workflow) => {
    const desired = !(overrides.get(w.slug) ?? w.enabled);
    setLocal(w.slug, desired);
    commit({
      variables: { slug: w.slug, enabled: desired },
      onError: () => setLocal(w.slug, !desired),
    });
  };

  const enabledCount = workflows.filter((w) => w.enabled).length;

  return (
    <div>
      <p className="text-muted-foreground mb-8 text-sm">
        {enabledCount} of {workflows.length} enabled
      </p>

      {dryRunError && (
        <p className="mb-6 text-sm text-red-600 dark:text-red-400">
          {dryRunError}
        </p>
      )}

      {groups.map(([group, list]) => (
        <section key={group} className="mb-10">
          <h2 className="text-muted-foreground mb-3 text-xs font-semibold tracking-widest uppercase">
            {group}
          </h2>
          <div className="flex flex-col gap-3">
            {list.map((w) => (
              <div
                key={w.slug}
                className="bg-card border-border flex items-center justify-between gap-4 rounded-2xl border p-4"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="truncate font-medium">{w.name}</span>
                    {w.dryRun && (
                      <span className="text-muted-foreground border-border rounded-full border px-1.5 py-0.5 text-[10px] tracking-wide uppercase">
                        dry run
                      </span>
                    )}
                    {w.reusable && (
                      <span className="text-muted-foreground border-border rounded-full border px-1.5 py-0.5 text-[10px] tracking-wide uppercase">
                        reusable
                      </span>
                    )}
                    {w.modes.map((mode) => (
                      <span
                        key={mode}
                        title="only fires in this mode"
                        className="rounded-full border border-sky-500/40 px-1.5 py-0.5 text-[10px] tracking-wide text-sky-600 uppercase dark:text-sky-400"
                      >
                        {mode.toLowerCase()}
                      </span>
                    ))}
                  </div>
                  <span className="text-muted-foreground font-mono text-xs">
                    {w.slug}
                  </span>
                </div>
                <div className="flex shrink-0 items-center gap-3">
                  <button
                    type="button"
                    onClick={() => dryRun(w)}
                    disabled={dryRunInFlight}
                    title="Run without side effects and show the trace"
                    className="border-border text-muted-foreground hover:text-foreground cursor-pointer rounded-full border px-3 py-1 text-xs transition-colors disabled:cursor-wait disabled:opacity-50"
                  >
                    Dry run
                  </button>
                  <Switch
                    checked={w.enabled}
                    onCheckedChange={() => toggle(w)}
                    aria-label={`Toggle ${w.name}`}
                  />
                </div>
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
