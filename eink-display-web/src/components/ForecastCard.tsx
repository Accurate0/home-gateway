import { graphql, useFragment } from "react-relay";
import type { ForecastCard_weather$key } from "./__generated__/ForecastCard_weather.graphql";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

const ForecastFragment = graphql`
  fragment ForecastCard_weather on WeatherObject {
    forecast {
      days {
        dateTime
        code
        description
        emoji
        min
        max
        uv
      }
    }
  }
`;

export default function ForecastCard({
  weatherRef,
}: {
  weatherRef: ForecastCard_weather$key;
}) {
  const data = useFragment(ForecastFragment, weatherRef);

  type ForecastObj = NonNullable<(typeof data)["forecast"]>;
  type DayType = NonNullable<ForecastObj["days"]>[number];

  const days = (data?.forecast?.days ?? []) as DayType[];
  const upcoming = days.slice(0, 4);

  return (
    <Card style={{ width: 600 }}>
      <CardHeader>
        <div>
          <CardTitle>Forecast</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(2, 1fr)",
            gap: 12,
          }}
        >
          {upcoming.map((d) => (
            <div
              key={d.dateTime}
              style={{
                borderRadius: 8,
                padding: 8,
                background: "transparent",
                display: "flex",
                gap: 8,
                alignItems: "center",
              }}
            >
              <div style={{ fontSize: 20 }}>{d.emoji ?? "☀️"}</div>
              <div style={{ display: "flex", flexDirection: "column" }}>
                <div style={{ fontWeight: 600 }}>
                  {formatForecastDate(d.dateTime)}
                </div>
                <div style={{ color: "#6b7280" }}>{d.description}</div>
                <div style={{ marginTop: 6, fontSize: 13 }}>
                  <span style={{ marginRight: 8 }}>Min: {d.min}°</span>
                  <span>Max: {d.max}°</span>
                  {d.uv != null && (
                    <span style={{ marginLeft: 8 }}>UV: {d.uv}</span>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

function formatForecastDate(dateTime: string) {
  try {
    const d = new Date(dateTime);
    const today = new Date();
    if (
      d.getFullYear() === today.getFullYear() &&
      d.getMonth() === today.getMonth() &&
      d.getDate() === today.getDate()
    ) {
      return "Today";
    }
    return d.toLocaleDateString();
  } catch {
    return dateTime;
  }
}
