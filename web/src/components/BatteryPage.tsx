import { Suspense, useEffect, useMemo, useState } from "react";
import {
  graphql,
  useQueryLoader,
  usePreloadedQuery,
  type PreloadedQuery,
} from "react-relay";
import { format } from "date-fns";
import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { BatteryPageQuery } from "./__generated__/BatteryPageQuery.graphql";
import { cn } from "@/lib/utils";

const BatteryQuery = graphql`
  query BatteryPageQuery($since: DateTime!) {
    entities {
      __typename
      ... on LightEntity {
        id
        name
        room
        battery {
          history(since: $since) {
            time
            batteryPercentage
          }
        }
      }
      ... on DoorEntity {
        id
        name
        room
        battery {
          history(since: $since) {
            time
            batteryPercentage
          }
        }
      }
      ... on PresenceEntity {
        id
        name
        room
        battery {
          history(since: $since) {
            time
            batteryPercentage
          }
        }
      }
      ... on EnvironmentEntity {
        id
        name
        room
        battery {
          history(since: $since) {
            time
            batteryPercentage
          }
        }
      }
      ... on EinkDisplayEntity {
        id
        name
        room
        battery {
          history(since: $since) {
            time
            batteryPercentage
          }
        }
      }
      ... on RobotVacuumEntity {
        id
        name
        room
        battery {
          history(since: $since) {
            time
            batteryPercentage
          }
        }
      }
    }
  }
`;

const DEFAULT_DAYS = 14;

const RANGES = [
  { label: "24h", days: 1 },
  { label: "7d", days: 7 },
  { label: "2w", days: 14 },
  { label: "30d", days: 30 },
  { label: "90d", days: 90 },
] as const;

const DEFAULT_RANGE = "2w";

const HOUR_MS = 60 * 60 * 1000;

function sinceFor(days: number) {
  const flooredNow = Math.floor(Date.now() / HOUR_MS) * HOUR_MS;
  return new Date(flooredNow - days * 24 * HOUR_MS).toISOString();
}

const SERIES_COLOURS = [
  "var(--series-1)",
  "var(--series-2)",
  "var(--series-3)",
  "var(--series-4)",
  "var(--series-5)",
  "var(--series-6)",
  "var(--series-7)",
  "var(--series-8)",
];

const SERIES_DASHES = ["", "6 4", "1 5"];

type Series = {
  id: string;
  label: string;
  colour: string;
  dash: string;
  points: { time: number; value: number }[];
};

type EntityRow = {
  readonly id?: string;
  readonly name?: string;
  readonly room?: string | null;
  readonly battery?: {
    readonly history: readonly {
      readonly time: string;
      readonly batteryPercentage?: number | null;
    }[];
  } | null;
};

function toSeries(entities: readonly EntityRow[]): Series[] {
  const withBattery = entities
    .filter((e) => e.id && e.battery && e.battery.history.length > 0)
    .sort((a, b) => a.id!.localeCompare(b.id!));

  return withBattery.map((e, i) => ({
    id: e.id!,
    label: e.room ? `${e.name} · ${e.room}` : (e.name ?? e.id!),
    colour: SERIES_COLOURS[i % SERIES_COLOURS.length],
    dash: SERIES_DASHES[Math.floor(i / SERIES_COLOURS.length) % SERIES_DASHES.length],
    points: e
      .battery!.history.filter((p) => p.batteryPercentage != null)
      .map((p) => ({
        time: new Date(p.time).getTime(),
        value: p.batteryPercentage!,
      }))
      .sort((a, b) => a.time - b.time),
  }));
}

function toChartRows(series: Series[]) {
  const times = [...new Set(series.flatMap((s) => s.points.map((p) => p.time)))].sort(
    (a, b) => a - b,
  );

  const cursors = new Map<string, number>();
  const last = new Map<string, number>();

  return times.map((time) => {
    const row: Record<string, number> = { time };

    for (const s of series) {
      let i = cursors.get(s.id) ?? 0;
      while (i < s.points.length && s.points[i].time <= time) {
        last.set(s.id, s.points[i].value);
        i += 1;
      }
      cursors.set(s.id, i);

      const value = last.get(s.id);
      if (value != null) row[s.id] = value;
    }

    return row;
  });
}

function tickFormatter(days: number) {
  const pattern = days <= 1 ? "HH:mm" : days <= 14 ? "d MMM" : "d MMM";
  return (value: number) => format(new Date(value), pattern);
}

function ChartTooltip({
  active,
  payload,
  label,
}: {
  active?: boolean;
  payload?: readonly {
    dataKey?: string | number;
    name?: string | number;
    value?: number;
    color?: string;
  }[];
  label?: number;
}) {
  if (!active || !payload?.length || label == null) return null;

  const rows = [...payload]
    .filter((p) => typeof p.value === "number")
    .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));

  return (
    <div className="bg-popover text-popover-foreground border-border rounded-lg border px-3 py-2 text-xs shadow-md">
      <p className="text-muted-foreground mb-1.5 font-medium">
        {format(new Date(label), "d MMM yyyy, HH:mm")}
      </p>
      <ul className="space-y-1">
        {rows.map((p) => (
          <li key={String(p.dataKey)} className="flex items-center gap-2">
            <span
              aria-hidden
              className="h-2 w-2 shrink-0 rounded-full"
              style={{ background: p.color }}
            />
            <span className="text-foreground">{String(p.name ?? p.dataKey)}</span>
            <span className="text-muted-foreground ml-auto tabular-nums">
              {Math.round(p.value ?? 0)}%
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}

type ChartProps = {
  queryRef: PreloadedQuery<BatteryPageQuery>;
  range: string;
  days: number;
  hidden: ReadonlySet<string>;
  showTable: boolean;
  onToggle: (id: string) => void;
};

function BatteryChart({
  queryRef,
  range,
  days,
  hidden,
  showTable,
  onToggle,
}: ChartProps) {
  const data = usePreloadedQuery<BatteryPageQuery>(BatteryQuery, queryRef);

  const series = useMemo(
    () => toSeries(data.entities as readonly EntityRow[]),
    [data.entities],
  );
  const visible = useMemo(() => series.filter((s) => !hidden.has(s.id)), [series, hidden]);
  const rows = useMemo(() => toChartRows(visible), [visible]);

  const toggle = onToggle;

  return (
    <div>
      {series.length === 0 ? (
        <p className="text-muted-foreground text-sm">
          No battery history in the last {range}.
        </p>
      ) : (
        <>
          <div className="h-[26rem] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={rows} margin={{ top: 8, right: 16, bottom: 8, left: 0 }}>
                <CartesianGrid
                  stroke="var(--border)"
                  strokeDasharray="3 3"
                  vertical={false}
                />
                <XAxis
                  dataKey="time"
                  type="number"
                  scale="time"
                  domain={["dataMin", "dataMax"]}
                  tickFormatter={tickFormatter(days)}
                  tick={{ fill: "var(--muted-foreground)", fontSize: 12 }}
                  stroke="var(--border)"
                  minTickGap={40}
                />
                <YAxis
                  domain={[0, 100]}
                  ticks={[0, 25, 50, 75, 100]}
                  tickFormatter={(v: number) => `${v}%`}
                  tick={{ fill: "var(--muted-foreground)", fontSize: 12 }}
                  stroke="var(--border)"
                  width={48}
                />
                <Tooltip
                  content={<ChartTooltip />}
                  cursor={{ stroke: "var(--muted-foreground)", strokeDasharray: "3 3" }}
                />
                {visible.map((s) => (
                  <Line
                    key={s.id}
                    type="monotone"
                    dataKey={s.id}
                    name={s.label}
                    stroke={s.colour}
                    strokeWidth={2}
                    strokeDasharray={s.dash || undefined}
                    dot={false}
                    activeDot={{ r: 4 }}
                    connectNulls
                    isAnimationActive={false}
                  />
                ))}
              </LineChart>
            </ResponsiveContainer>
          </div>

          <ul className="mt-6 flex flex-wrap gap-x-4 gap-y-2">
            {series.map((s) => {
              const off = hidden.has(s.id);
              return (
                <li key={s.id}>
                  <button
                    type="button"
                    onClick={() => toggle(s.id)}
                    aria-pressed={!off}
                    className={cn(
                      "flex cursor-pointer items-center gap-2 text-xs transition-opacity",
                      off ? "opacity-40" : "opacity-100",
                    )}
                  >
                    <svg aria-hidden width="18" height="8" className="shrink-0">
                      <line
                        x1="0"
                        y1="4"
                        x2="18"
                        y2="4"
                        stroke={s.colour}
                        strokeWidth="2"
                        strokeDasharray={s.dash || undefined}
                      />
                    </svg>
                    <span className="text-foreground">{s.label}</span>
                  </button>
                </li>
              );
            })}
          </ul>

          {showTable && (
            <div className="border-border mt-8 overflow-x-auto rounded-lg border">
              <table className="w-full text-left text-xs">
                <caption className="sr-only">
                  Latest battery percentage per device over the last {range}
                </caption>
                <thead className="text-muted-foreground border-border border-b">
                  <tr>
                    <th scope="col" className="px-3 py-2 font-medium">
                      Device
                    </th>
                    <th scope="col" className="px-3 py-2 font-medium">
                      Latest
                    </th>
                    <th scope="col" className="px-3 py-2 font-medium">
                      Min
                    </th>
                    <th scope="col" className="px-3 py-2 font-medium">
                      Max
                    </th>
                    <th scope="col" className="px-3 py-2 font-medium">
                      Readings
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {series.map((s) => {
                    const values = s.points.map((p) => p.value);
                    return (
                      <tr key={s.id} className="border-border border-b last:border-0">
                        <th scope="row" className="text-foreground px-3 py-2 font-normal">
                          {s.label}
                        </th>
                        <td className="px-3 py-2 tabular-nums">
                          {Math.round(values[values.length - 1])}%
                        </td>
                        <td className="px-3 py-2 tabular-nums">
                          {Math.round(Math.min(...values))}%
                        </td>
                        <td className="px-3 py-2 tabular-nums">
                          {Math.round(Math.max(...values))}%
                        </td>
                        <td className="text-muted-foreground px-3 py-2 tabular-nums">
                          {values.length}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </>
      )}
    </div>
  );
}

export default function BatteryPage() {
  const [range, setRange] = useState<string>(DEFAULT_RANGE);
  const [since, setSince] = useState(() => sinceFor(DEFAULT_DAYS));
  const [hidden, setHidden] = useState<ReadonlySet<string>>(new Set());
  const [showTable, setShowTable] = useState(false);

  const [queryRef, loadQuery] = useQueryLoader<BatteryPageQuery>(BatteryQuery);

  useEffect(() => {
    loadQuery({ since }, { fetchPolicy: "store-or-network" });
  }, [loadQuery, since]);

  const days = RANGES.find((r) => r.label === range)?.days ?? DEFAULT_DAYS;

  function toggle(id: string) {
    setHidden((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  return (
    <div>
      <div className="mb-6 flex flex-wrap items-center gap-2">
        {RANGES.map((r) => (
          <button
            key={r.label}
            type="button"
            onClick={() => {
              setRange(r.label);
              setSince(sinceFor(r.days));
            }}
            className={cn(
              "cursor-pointer rounded-full border px-3 py-1 text-xs transition-colors",
              range === r.label
                ? "border-foreground text-foreground"
                : "border-border text-muted-foreground hover:text-foreground",
            )}
          >
            {r.label}
          </button>
        ))}

        <button
          type="button"
          onClick={() => setShowTable((v) => !v)}
          className="border-border text-muted-foreground hover:text-foreground ml-auto cursor-pointer rounded-full border px-3 py-1 text-xs transition-colors"
        >
          {showTable ? "Hide table" : "Show table"}
        </button>
      </div>

      <Suspense
        fallback={<p className="text-muted-foreground text-sm">Loading…</p>}
      >
        {queryRef && (
          <BatteryChart
            queryRef={queryRef}
            range={range}
            days={days}
            hidden={hidden}
            showTable={showTable}
            onToggle={toggle}
          />
        )}
      </Suspense>
    </div>
  );
}
