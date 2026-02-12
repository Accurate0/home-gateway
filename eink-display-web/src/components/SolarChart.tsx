import { graphql, useFragment } from "react-relay";
import type { SolarChart_solar$key } from "./__generated__/SolarChart_solar.graphql";
import * as Recharts from "recharts";
import { ChartContainer } from "@/components/ui/chart";

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

export default function SolarChart({
  solarRef,
  width,
  height,
}: {
  solarRef: SolarChart_solar$key;
  width?: number;
  height?: number;
}) {
  const data = useFragment(SolarFragment, solarRef);
  if (!data) {
    return <div>No data</div>;
  }

  const history = (data.history ?? []).map((h) => ({
    wh: h.wh,
    at: h.at,
    timestamp: h.timestamp,
    uv: h.uvLevel ?? 0,
  }));

  history.sort((a, b) => a.timestamp - b.timestamp);

  const chartData = history.map((h) => ({ ...h, atLabel: formatTime(h.at) }));

  return (
    <div style={{ width: width ?? 740, height: height ?? 600 }}>
      <ChartContainer
        id="solar"
        config={{
          wh: { label: "Wh", color: "#0000ff" },
          uv: { label: "UV", color: "#dc2626" },
        }}
      >
        <Recharts.LineChart
          width={width ?? 740}
          height={height ?? 600}
          data={chartData}
          margin={{ top: 10, right: 40, left: 10, bottom: 40 }}
        >
          <Recharts.CartesianGrid
            strokeDasharray="3 3"
            vertical={false}
            stroke="#ccc"
          />
          <Recharts.XAxis
            dataKey="atLabel"
            tick={{ fontSize: 20, fill: "black" }}
            interval={Math.floor(chartData.length / 6)}
            tickFormatter={(value: Date) => {
              const newDate = new Date(value);
              return new Intl.DateTimeFormat("en-AU", {
                timeZone: "Australia/Perth",
                hour: "2-digit",
                minute: "2-digit",
                hour12: false,
              }).format(newDate);
            }}
            dy={15}
          />

          <Recharts.YAxis
            yAxisId="left"
            tick={{ fontSize: 20, fill: "black" }}
            width={80}
            tickFormatter={(value) => `${value}W`}
          />
          <Recharts.YAxis
            yAxisId="right"
            orientation="right"
            tick={{ fontSize: 20, fill: "#dc2626" }}
            width={50}
            domain={[0, "auto"]}
            tickFormatter={(value) => `${value}`}
          />
          <Recharts.Line
            yAxisId="left"
            type="monotone"
            dataKey="wh"
            stroke="#0000ff"
            dot={false}
            strokeWidth={6}
            animationDuration={0}
          />
          <Recharts.Line
            yAxisId="right"
            type="monotone"
            dataKey="uv"
            stroke="#dc2626"
            dot={false}
            strokeWidth={4}
            strokeDasharray="5 5"
            animationDuration={0}
          />
        </Recharts.LineChart>
      </ChartContainer>
    </div>
  );
}

function formatTime(at: string) {
  try {
    const d = new Date(at);
    return d;
  } catch {
    return at;
  }
}
