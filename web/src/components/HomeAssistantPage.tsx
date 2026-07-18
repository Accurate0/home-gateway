import { useEffect, useMemo, useState } from "react";
import { graphql, useLazyLoadQuery, useRelayEnvironment } from "react-relay";
import { requestSubscription } from "relay-runtime";
import { formatDistanceToNow } from "date-fns";
import type { HomeAssistantPageQuery } from "./__generated__/HomeAssistantPageQuery.graphql";
import type { HomeAssistantPageSubscription } from "./__generated__/HomeAssistantPageSubscription.graphql";
import { cn } from "@/lib/utils";

const HistoryQuery = graphql`
  query HomeAssistantPageQuery($since: DateTime!) {
    events(input: { since: $since }) {
      homeAssistant {
        eventId
        entityId
        state
        time
      }
    }
  }
`;

const UpdatesSubscription = graphql`
  subscription HomeAssistantPageSubscription {
    events(filter: "home_assistant:*") {
      __typename
      ... on HomeAssistantUpdate {
        eventId
        state
        entityId
      }
    }
  }
`;

const MAX_ROWS = 500;

type Row = {
  eventId: string;
  entityId: string;
  state: string;
  time: string;
};

export default function HomeAssistantPage() {
  const [since] = useState(() =>
    new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
  );
  const data = useLazyLoadQuery<HomeAssistantPageQuery>(HistoryQuery, { since });
  const environment = useRelayEnvironment();
  const [rows, setRows] = useState<Row[]>(() =>
    data.events.homeAssistant.map((e) => ({
      eventId: e.eventId,
      entityId: e.entityId,
      state: e.state,
      time: e.time as string,
    })),
  );
  const [filter, setFilter] = useState("");

  useEffect(() => {
    const sub = requestSubscription<HomeAssistantPageSubscription>(environment, {
      subscription: UpdatesSubscription,
      variables: {},
      onNext: (response) => {
        const update = response?.events;
        if (!update || update.__typename !== "HomeAssistantUpdate") return;
        const row: Row = {
          eventId: update.eventId,
          entityId: update.entityId,
          state: update.state,
          time: new Date().toISOString(),
        };
        setRows((prev) => [row, ...prev].slice(0, MAX_ROWS));
      },
    });
    return () => sub.dispose();
  }, [environment]);

  const visible = useMemo(
    () =>
      filter
        ? rows.filter((r) =>
            r.entityId.toLowerCase().includes(filter.toLowerCase()),
          )
        : rows,
    [rows, filter],
  );

  return (
    <div>
      <p className="text-muted-foreground mb-6 flex items-center gap-2 text-sm">
        <span className="bg-state-present relative flex size-2 rounded-full">
          <span className="bg-state-present absolute inline-flex size-full animate-ping rounded-full opacity-75" />
        </span>
        {rows.length} updates · live
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
            <div
              key={row.eventId}
              className="bg-card border-border flex items-center justify-between gap-4 rounded-2xl border p-4"
            >
              <span className="min-w-0 truncate font-mono text-xs">
                {row.entityId}
              </span>
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
          ))}
        </div>
      )}
    </div>
  );
}
