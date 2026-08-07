import { graphql, useFragment } from "react-relay";
import type { SolarChart_solar$key } from "./__generated__/SolarChart_solar.graphql";
import * as Recharts from "recharts";
import { ChartContainer } from "@/components/ui/chart";
import { BLUE, INK, RED, TYPE } from "../theme";

const SolarFragment = graphql`
  fragment SolarChart_solar on SolarObject
  @argumentDefinitions(since: { type: "DateTime!" }) {
    history(input: { since: $since }) {
      wh
      at
      timestamp
      uvLevel
    }
  }
`;

const TICK = { fontSize: 22, fontWeight: 700 };
const TARGET_TICKS = 6;

const timeFormatter = new Intl.DateTimeFormat("en-AU", {
  timeZone: "Australia/Perth",
  hour: "2-digit",
  minute: "2-digit",
  hour12: false,
});

type Point = { wh: number; uv: number; at: number };

const EDGE_PADDING = 3;

function trimToDaylight(points: Point[]) {
  const active = points.map((p) => p.wh > 0 || p.uv > 0);
  const first = active.indexOf(true);
  if (first === -1) return points;

  const last = active.lastIndexOf(true);

  return points.slice(
    Math.max(0, first - EDGE_PADDING),
    Math.min(points.length, last + 1 + EDGE_PADDING),
  );
}

export default function SolarChart({
  solarRef,
  width,
  height,
}: {
  solarRef: SolarChart_solar$key;
  width: number;
  height: number;
}) {
  const data = useFragment(SolarFragment, solarRef);

  const all = [...(data?.history ?? [])]
    .sort((a, b) => a.timestamp - b.timestamp)
    .map((h) => ({
      wh: h.wh,
      uv: h.uvLevel ?? 0,
      at: Date.parse(`${h.at}Z`),
    }));

  const history = trimToDaylight(all);

  if (history.length === 0) {
    return (
      <div
        style={{
          width,
          height,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          ...TYPE.label,
        }}
      >
        No generation data yet
      </div>
    );
  }

  const interval = Math.max(1, Math.floor(history.length / TARGET_TICKS));

  return (
    <div style={{ width, height }}>
      <ChartContainer
        id="solar"
        className="aspect-auto h-full w-full"
        config={{
          wh: { label: "Wh", color: BLUE },
          uv: { label: "UV", color: RED },
        }}
      >
        <Recharts.LineChart
          width={width}
          height={height}
          data={history}
          margin={{ top: 8, right: 8, left: 0, bottom: 24 }}
        >
          <Recharts.CartesianGrid vertical={false} stroke={INK} strokeWidth={2} />
          <Recharts.XAxis
            dataKey="at"
            tick={{ ...TICK, fill: INK }}
            tickLine={{ stroke: INK, strokeWidth: 2 }}
            axisLine={{ stroke: INK, strokeWidth: 3 }}
            interval={interval}
            tickFormatter={(value: number) => timeFormatter.format(new Date(value))}
            dy={12}
          />

          <Recharts.YAxis
            yAxisId="left"
            tick={{ ...TICK, fill: INK }}
            tickLine={{ stroke: INK, strokeWidth: 2 }}
            axisLine={{ stroke: INK, strokeWidth: 3 }}
            width={92}
            tickFormatter={(value: number) => `${value}W`}
          />
          <Recharts.YAxis
            yAxisId="right"
            orientation="right"
            tick={{ ...TICK, fill: RED }}
            tickLine={{ stroke: INK, strokeWidth: 2 }}
            axisLine={{ stroke: INK, strokeWidth: 3 }}
            width={56}
            domain={[0, "auto"]}
          />

          <Recharts.Line
            yAxisId="left"
            type="monotone"
            dataKey="wh"
            stroke={BLUE}
            dot={false}
            strokeWidth={6}
            animationDuration={0}
          />
          <Recharts.Line
            yAxisId="right"
            type="monotone"
            dataKey="uv"
            stroke={RED}
            dot={false}
            strokeWidth={4}
            animationDuration={0}
          />
        </Recharts.LineChart>
      </ChartContainer>
    </div>
  );
}
