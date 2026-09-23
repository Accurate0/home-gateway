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
  ReferenceArea,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { PlantsPageQuery } from "./__generated__/PlantsPageQuery.graphql";
import { cn } from "@/lib/utils";

const PlantQuery = graphql`
  query PlantsPageQuery($since: DateTime!) {
    entities {
      __typename
      ... on PlantEntity {
        id
        name
        room
        soilMoisture
        time
        history(since: $since) {
          time
          soilMoisture
        }
      }
    }
  }
`;

const DRY = 20;
const WET = 80;

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

type Series = {
  id: string;
  label: string;
  colour: string;
  latest: number | null;
  time: string | null;
  points: { time: number; value: number }[];
};

type EntityRow = {
  readonly id?: string;
  readonly name?: string;
  readonly room?: string | null;
  readonly soilMoisture?: number | null;
  readonly time?: string | null;
  readonly history?: readonly {
    readonly time: string;
    readonly soilMoisture: number;
  }[];
};

function toSeries(entities: readonly EntityRow[]): Series[] {
  return entities
    .filter((e) => e.id && e.history)
    .sort((a, b) => a.id!.localeCompare(b.id!))
    .map((e, i) => ({
      id: e.id!,
      label: e.room ? `${e.name} · ${e.room}` : (e.name ?? e.id!),
      colour: SERIES_COLOURS[i % SERIES_COLOURS.length],
      latest: e.soilMoisture ?? null,
      time: e.time ?? null,
      points: e
        .history!.map((p) => ({
          time: new Date(p.time).getTime(),
          value: p.soilMoisture,
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
  const pattern = days <= 1 ? "HH:mm" : "d MMM";
  return (value: number) => format(new Date(value), pattern);
}

function statusOf(value: number | null) {
  if (value == null) return "-";
  if (value < DRY) return "dry";
  if (value > WET) return "overwatered";
  return "ok";
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

function PlantChart({
  queryRef,
  range,
  days,
  hidden,
  onToggle,
}: {
  queryRef: PreloadedQuery<PlantsPageQuery>;
  range: string;
  days: number;
  hidden: ReadonlySet<string>;
  onToggle: (id: string) => void;
}) {
  const data = usePreloadedQuery<PlantsPageQuery>(PlantQuery, queryRef);

  const series = useMemo(
    () => toSeries(data.entities as readonly EntityRow[]),
    [data.entities],
  );
  const visible = useMemo(() => series.filter((s) => !hidden.has(s.id)), [series, hidden]);
  const rows = useMemo(() => toChartRows(visible), [visible]);

  if (series.length === 0) {
    return <p className="text-muted-foreground text-sm">No plants configured.</p>;
  }

  const hasHistory = series.some((s) => s.points.length > 0);

  return (
    <div>
      <ul className="mb-8 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {series.map((s) => (
          <li
            key={s.id}
            className="border-border rounded-2xl border p-4"
          >
            <div className="flex items-start justify-between">
              <div>
                <div className="leading-tight font-medium">{s.label}</div>
                <div className="text-muted-foreground text-xs">
                  {s.time ? format(new Date(s.time), "d MMM, HH:mm") : "no reading"}
                </div>
              </div>
              <div className="font-display text-3xl font-semibold tracking-tight tabular-nums">
                {s.latest == null ? "-" : `${Math.round(s.latest)}%`}
              </div>
            </div>
            <div className="mt-4">
              <div className="text-muted-foreground mb-1.5 flex justify-between text-xs">
                <span>Soil moisture</span>
                <span>{statusOf(s.latest)}</span>
              </div>
              <div className="bg-muted h-1.5 overflow-hidden rounded-full">
                <div
                  className={cn(
                    "h-full rounded-full transition-[width]",
                    s.latest == null
                      ? "bg-muted-foreground/40"
                      : s.latest < DRY || s.latest > WET
                        ? "bg-state-open/70"
                        : "bg-state-present/70",
                  )}
                  style={{
                    width: `${s.latest == null ? 0 : Math.max(0, Math.min(100, s.latest))}%`,
                  }}
                />
              </div>
            </div>
          </li>
        ))}
      </ul>

      {!hasHistory ? (
        <p className="text-muted-foreground text-sm">
          No soil moisture history in the last {range}.
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
                <ReferenceArea
                  y1={0}
                  y2={DRY}
                  fill="var(--state-open)"
                  fillOpacity={0.08}
                />
                <ReferenceArea
                  y1={WET}
                  y2={100}
                  fill="var(--state-open)"
                  fillOpacity={0.08}
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
                    onClick={() => onToggle(s.id)}
                    aria-pressed={!off}
                    className={cn(
                      "flex cursor-pointer items-center gap-2 text-xs transition-opacity",
                      off ? "opacity-40" : "opacity-100",
                    )}
                  >
                    <svg aria-hidden width="18" height="8" className="shrink-0">
                      <line x1="0" y1="4" x2="18" y2="4" stroke={s.colour} strokeWidth="2" />
                    </svg>
                    <span className="text-foreground">{s.label}</span>
                  </button>
                </li>
              );
            })}
          </ul>
        </>
      )}
    </div>
  );
}

export default function PlantsPage() {
  const [range, setRange] = useState<string>(DEFAULT_RANGE);
  const [since, setSince] = useState(() => sinceFor(DEFAULT_DAYS));
  const [hidden, setHidden] = useState<ReadonlySet<string>>(new Set());

  const [queryRef, loadQuery] = useQueryLoader<PlantsPageQuery>(PlantQuery);

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
      </div>

      <Suspense fallback={<p className="text-muted-foreground text-sm">Loading…</p>}>
        {queryRef && (
          <PlantChart
            queryRef={queryRef}
            range={range}
            days={days}
            hidden={hidden}
            onToggle={toggle}
          />
        )}
      </Suspense>
    </div>
  );
}
