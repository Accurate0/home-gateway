import { graphql, useFragment } from "react-relay";
import type { ForecastCard_weather$key } from "./__generated__/ForecastCard_weather.graphql";

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
  const upcoming = days.slice(0, 4); // Show 4 days

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 24,
      }}
    >
      {upcoming.map((d, i) => (
        <div
          key={d.dateTime}
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "16px 0",
            borderBottom: i === upcoming.length - 1 ? "none" : "2px solid #eee",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 24 }}>
            <div style={{ fontSize: 64, width: 80, textAlign: "center", display: "flex", justifyContent: "center" }}>
              {d.emoji ?? "☀️"}
            </div>
            <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-start", lineHeight: 1.2 }}>
              <div style={{ fontSize: 32, fontWeight: 700 }}>
                {formatForecastDate(d.dateTime)}
              </div>
              <div style={{ fontSize: 24, color: "#4b5563" }}>
                {d.description}
              </div>
            </div>
          </div>
          <div style={{ textAlign: "right" }}>
            <div style={{ fontSize: 32, fontWeight: 700 }}>
              {d.max}° <span style={{ color: "#6b7280", fontWeight: 400 }}>{d.min}°</span>
            </div>
            {d.uv != null && (
              <div style={{ fontSize: 20, color: "#ef4444", fontWeight: 600 }}>
                UV {d.uv.toFixed(1)}
              </div>
            )}
          </div>
        </div>
      ))}
    </div>
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
