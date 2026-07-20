import { memo, useEffect, useMemo, useState } from "react";
import { graphql, useRelayEnvironment } from "react-relay";
import { fetchQuery, requestSubscription } from "relay-runtime";
import { formatDistanceToNow } from "date-fns";
import type { HomeAssistantPageQuery } from "./__generated__/HomeAssistantPageQuery.graphql";
import type { HomeAssistantPageSubscription } from "./__generated__/HomeAssistantPageSubscription.graphql";
import { cn } from "@/lib/utils";

const HistoryQuery = graphql`
  query HomeAssistantPageQuery {
    homeAssistantEntities {
      id
      eventId
      entityId
      state
      time
    }
  }
`;

const UpdatesSubscription = graphql`
  subscription HomeAssistantPageSubscription {
    events(filter: "home_assistant:*") {
      __typename
      ... on HomeAssistantUpdate {
        id
        eventId
        state
        entityId
      }
    }
  }
`;

type Row = {
  entityId: string;
  state: string;
  time: string;
};

const HistoryRow = memo(function HistoryRow({ row }: { row: Row }) {
  return (
    <div className="bg-card border-border flex items-center justify-between gap-4 rounded-2xl border p-4">
      <span className="min-w-0 truncate font-mono text-xs">{row.entityId}</span>
      <div className="flex shrink-0 flex-col items-end gap-1 text-right">
        <span
          className={cn(
            "border-border rounded-full border px-2 py-0.5 font-mono text-xs",
          )}
        >
          {row.state}
        </span>
        <span className="text-muted-foreground text-xs">
          {formatDistanceToNow(new Date(row.time), { addSuffix: true })}
        </span>
      </div>
    </div>
  );
});

function upsert(prev: Map<string, Row>, row: Row): Map<string, Row> {
  const existing = prev.get(row.entityId);
  if (existing && existing.time >= row.time) return prev;
  const next = new Map(prev);
  next.set(row.entityId, row);
  return next;
}

export default function HomeAssistantPage() {
  const environment = useRelayEnvironment();
  const [latest, setLatest] = useState<Map<string, Row>>(() => new Map());
  const [filter, setFilter] = useState("");

  useEffect(() => {
    const sub = fetchQuery<HomeAssistantPageQuery>(
      environment,
      HistoryQuery,
      {},
    ).subscribe({
      next: (data) => {
        setLatest((prev) =>
          data.homeAssistantEntities.reduce(
            (acc, e) =>
              upsert(acc, {
                entityId: e.entityId,
                state: e.state,
                time: e.time as string,
              }),
            prev,
          ),
        );
      },
    });
    return () => sub.unsubscribe();
  }, [environment]);

  useEffect(() => {
    const sub = requestSubscription<HomeAssistantPageSubscription>(environment, {
      subscription: UpdatesSubscription,
      variables: {},
      onNext: (response) => {
        const update = response?.events;
        if (!update || update.__typename !== "HomeAssistantUpdate") return;
        setLatest((prev) =>
          upsert(prev, {
            entityId: update.entityId,
            state: update.state,
            time: new Date().toISOString(),
          }),
        );
      },
    });
    return () => sub.dispose();
  }, [environment]);

  const visible = useMemo(() => {
    const rows = [...latest.values()].sort((a, b) =>
      b.time.localeCompare(a.time),
    );
    return filter
      ? rows.filter((r) => r.entityId.toLowerCase().includes(filter.toLowerCase()))
      : rows;
  }, [latest, filter]);

  return (
    <div>
      <p className="text-muted-foreground mb-6 flex items-center gap-2 text-sm">
        <span className="bg-state-present relative flex size-2 rounded-full">
          <span className="bg-state-present absolute inline-flex size-full animate-ping rounded-full opacity-75" />
        </span>
        {latest.size} entities · live
      </p>

      <input
        type="text"
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        placeholder="Filter by entity id…"
        className="border-border bg-card mb-6 w-full max-w-md rounded-full border px-4 py-1.5 font-mono text-xs outline-none focus:border-foreground"
      />

      {visible.length === 0 ? (
        <p className="text-muted-foreground text-sm">
          No Home Assistant updates recorded yet.
        </p>
      ) : (
        <div className="flex flex-col gap-2">
          {visible.map((row) => (
            <HistoryRow key={row.entityId} row={row} />
          ))}
        </div>
      )}
    </div>
  );
}
