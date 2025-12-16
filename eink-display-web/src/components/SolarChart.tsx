import { graphql, useLazyLoadQuery } from "react-relay";
import type { SolarChartQuery } from "./__generated__/SolarChartQuery.graphql";
import * as Recharts from "recharts";
import { ChartContainer } from "@/components/ui/chart";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

type GenerationHistory = {
  wh: number;
  at: string;
  timestamp: number;
};

const SolarQuery = graphql`
  query SolarChartQuery($since: DateTime!) {
    solar(input: { since: $since }) {
      history {
        wh
        at
        timestamp
      }
    }
  }
`;

export default function SolarChart() {
  // start of today in RFC3339
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const since = today.toISOString();

  const data = useLazyLoadQuery<SolarChartQuery>(
    SolarQuery,
    { since },
    { fetchPolicy: "store-or-network" }
  );

  const historyRaw = data?.solar?.history ?? [];
  type Gen = { wh: number; at: string; timestamp: number };
  const history: GenerationHistory[] = (historyRaw as unknown as Gen[]).map(
    (h) => ({
      wh: h.wh,
      at: h.at,
      timestamp: h.timestamp,
    })
  );

  // sort by timestamp
  history.sort((a, b) => a.timestamp - b.timestamp);

  // convert `at` to a JS Date string for x-axis
  const chartData = history.map((h) => ({ ...h, atLabel: formatTime(h.at) }));

  return (
    <div style={{}}>
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
        <Card style={{ width: "100%", height: "100%" }}>
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
                      <Recharts.YAxis tick={{ fontSize: 12 }} />
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
