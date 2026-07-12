import { useMemo, useState } from "react";
import { graphql, useLazyLoadQuery, useMutation } from "react-relay";
import type { WorkflowsPageQuery } from "./__generated__/WorkflowsPageQuery.graphql";
import type { WorkflowsPageSetEnabledMutation } from "./__generated__/WorkflowsPageSetEnabledMutation.graphql";
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
    }
  }
`;

const SetEnabledMutation = graphql`
  mutation WorkflowsPageSetEnabledMutation($slug: String!, $enabled: Boolean!) {
    setWorkflowEnabled(slug: $slug, enabled: $enabled)
  }
`;

type Workflow = WorkflowsPageQuery["response"]["workflows"][number];

export default function WorkflowsPage() {
  const data = useLazyLoadQuery<WorkflowsPageQuery>(WorkflowsQuery, {});
  const [overrides, setOverrides] = useState<Map<string, boolean>>(new Map());
  const [commit] = useMutation<WorkflowsPageSetEnabledMutation>(
    SetEnabledMutation,
  );

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
                  </div>
                  <span className="text-muted-foreground font-mono text-xs">
                    {w.slug}
                  </span>
                </div>
                <Switch
                  checked={w.enabled}
                  onCheckedChange={() => toggle(w)}
                  aria-label={`Toggle ${w.name}`}
                />
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
