import { graphql, useFragment } from "react-relay";
import type { ForecastCard_weather$key } from "./__generated__/ForecastCard_weather.graphql";
import { Sun, Cloud, CloudRain, CloudLightning, CloudSnow, CloudFog, CloudSun } from "lucide-react";

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

function WeatherIcon({ code, size = 80 }: { code: string; size?: number }) {
  const c = code.toLowerCase();
  const props = { size, strokeWidth: 2, fill: "currentColor" };
  
  if (c.includes("sunny") || c.includes("clear")) return <Sun {...props} color="#f59e0b" fill="#f59e0b" />;
  if (c.includes("partly") || c.includes("mostly sunny")) return <CloudSun {...props} color="#f59e0b" fill="#f59e0b" />;
  if (c.includes("cloudy")) return <Cloud {...props} color="#6b7280" fill="#6b7280" />;
  if (c.includes("rain") || c.includes("shower")) return <CloudRain {...props} color="#3b82f6" fill="#3b82f6" />;
  if (c.includes("storm") || c.includes("thunder")) return <CloudLightning {...props} color="#7c3aed" fill="#7c3aed" />;
  if (c.includes("snow")) return <CloudSnow {...props} color="#0ea5e9" fill="#0ea5e9" />;
  if (c.includes("fog") || c.includes("mist")) return <CloudFog {...props} color="#94a3b8" fill="#94a3b8" />;
  return <Sun {...props} color="#f59e0b" fill="#f59e0b" />;
}

export default function ForecastCard({
  weatherRef,
}: {
  weatherRef: ForecastCard_weather$key;
}) {
  const data = useFragment(ForecastFragment, weatherRef);

  type ForecastObj = NonNullable<(typeof data)["forecast"]>;
  type DayType = NonNullable<ForecastObj["days"]>[number];

  const days = (data?.forecast?.days ?? []) as DayType[];
  const upcoming = days.slice(0, 6); // Show 6 days

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
            padding: "20px 0",
            borderBottom: i === upcoming.length - 1 ? "none" : "3px solid #eee",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 24 }}>
            <div style={{ width: 80, display: "flex", justifyContent: "center" }}>
              <WeatherIcon code={d.code} size={72} />
            </div>
            <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-start", lineHeight: 1.1 }}>
              <div style={{ fontSize: 36, fontWeight: 800 }}>
                {formatForecastDate(d.dateTime)}
              </div>
              <div style={{ fontSize: 28, fontWeight: 500, color: "#4b5563" }}>
                {d.description}
              </div>
            </div>
          </div>
          <div style={{ textAlign: "right" }}>
            <div style={{ fontSize: 36, fontWeight: 800 }}>
              {d.max}° <span style={{ color: "#6b7280", fontWeight: 500 }}>{d.min}°</span>
            </div>
            {d.uv != null && (
              <div style={{ fontSize: 22, color: "#dc2626", fontWeight: 800, marginTop: 4 }}>
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
    today.setHours(0, 0, 0, 0);
    
    const compareDate = new Date(d);
    compareDate.setHours(0, 0, 0, 0);

    const diffTime = compareDate.getTime() - today.getTime();
    const diffDays = Math.round(diffTime / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "Today";
    if (diffDays === 1) return "Tomorrow";
    
    return d.toLocaleDateString([], { day: "numeric", month: "short" });
  } catch {
    return dateTime;
  }
}
