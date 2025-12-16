import { graphql, useFragment } from "react-relay";
import type { SolarChart_solar$key } from "./__generated__/SolarChart_solar.graphql";
import * as Recharts from "recharts";
import { ChartContainer } from "@/components/ui/chart";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

const SolarFragment = graphql`
  fragment SolarChart_solar on SolarObject {
    history {
      wh
      at
      timestamp
    }
  }
`;

export default function SolarChart({
  solarRef,
}: {
  solarRef: SolarChart_solar$key;
}) {
  const data = useFragment(SolarFragment, solarRef);
  if (!data) {
    return (
      <Card>
        <CardHeader>
          <div>
            <CardTitle>Solar</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <div>
            No fragment data received for solar. Check parent query/relay
            artifacts.
          </div>
        </CardContent>
      </Card>
    );
  }

  const history = (data.history ?? []).map((h) => ({
    wh: h.wh,
    at: h.at,
    timestamp: h.timestamp,
  }));

  // sort by timestamp
  history.sort((a, b) => a.timestamp - b.timestamp);

  // convert `at` to a JS Date string for x-axis
  const chartData = history.map((h) => ({ ...h, atLabel: formatTime(h.at) }));

  return (
    <div>
      <div
        style={{
          width: "100%",
          height: "100%",
          boxSizing: "border-box",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <Card style={{ width: 900, height: 650 }}>
          <CardHeader>
            <div>
              <CardTitle>Solar</CardTitle>
            </div>
          </CardHeader>
          <CardContent
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 12,
              padding: 12,
              height: "100%",
            }}
          >
            <div
              style={{
                width: "100%",
                height: "100%",
                display: "flex",
                flexDirection: "column",
              }}
            >
              <div style={{ flex: 1, padding: 8 }}>
                <ChartContainer
                  id="solar"
                  config={{ wh: { label: "Wh", color: "#06b6d4" } }}
                >
                  <Recharts.ResponsiveContainer width="100%" height="100%">
                    <Recharts.LineChart
                      data={chartData}
                      margin={{ top: 6, right: 16, left: 6, bottom: 20 }}
                    >
                      <Recharts.YAxis tick={{ fontSize: 16 }} />
                      <Recharts.Line
                        type="monotone"
                        dataKey="wh"
                        stroke="var(--color-wh, #06b6d4)"
                        dot={false}
                        strokeWidth={2}
                      />
                    </Recharts.LineChart>
                  </Recharts.ResponsiveContainer>
                </ChartContainer>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function formatTime(at: string) {
  try {
    const d = new Date(at);
    return `${d.getHours()}:${String(d.getMinutes()).padStart(2, "0")}`;
  } catch {
    return at;
  }
}
