import { graphql, useLazyLoadQuery } from "react-relay";
import type { ForecastCardQuery } from "./__generated__/ForecastCardQuery.graphql";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

const ForecastQuery = graphql`
  query ForecastCardQuery($location: String!) {
    weather(input: { location: $location }) {
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
  }
`;

export default function ForecastCard({
  location = "14576",
}: {
  location?: string;
}) {
  const data = useLazyLoadQuery<ForecastCardQuery>(ForecastQuery, { location });

  // Strongly-typed day element derived from generated types
  type Response = ForecastCardQuery["response"];
  type WeatherObj = NonNullable<Response["weather"]>;
  type ForecastObj = NonNullable<WeatherObj["forecast"]>;
  type DayType = NonNullable<ForecastObj["days"]>[number];

  const days = (data?.weather?.forecast?.days ?? []) as DayType[];

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
          {days.map((d) => (
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
                  {new Date(d.dateTime).toLocaleDateString()}
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
