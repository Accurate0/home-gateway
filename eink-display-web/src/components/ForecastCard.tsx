import { graphql, useFragment } from "react-relay";
import type { ForecastCard_weather$key } from "./__generated__/ForecastCard_weather.graphql";
import WeatherIcon from "./WeatherIcon";
import { fromToday } from "../lib/time";
import { HIGH_UV, INK, PAPER, RED, TYPE } from "../theme";

const ForecastFragment = graphql`
  fragment ForecastCard_weather on WeatherObject {
    forecast {
      days {
        dateTime
        code
        description
        min
        max
        uv
      }
    }
  }
`;

export default function ForecastCard({
  weatherRef,
  height,
  count = 3,
}: {
  weatherRef: ForecastCard_weather$key;
  height: number;
  count?: number;
}) {
  const data = useFragment(ForecastFragment, weatherRef);
  const days = fromToday(data?.forecast?.days ?? []).slice(1, 1 + count);

  const rowHeight = height / count;
  const iconSize = Math.min(80, rowHeight - 24);

  return (
    <section style={{ height }}>
      {days.map((d, i) => (
        <div
          key={d.dateTime}
          style={{
            height: rowHeight,
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            borderBottom: i === days.length - 1 ? "none" : `2px solid ${INK}`,
            boxSizing: "border-box",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 24, flex: 1 }}>
            <WeatherIcon code={d.code} size={iconSize} />
            <div>
              <div style={TYPE.body}>{formatForecastDate(d.dateTime)}</div>
              <div style={{ fontSize: 24, fontWeight: 600, marginTop: 2 }}>{d.description}</div>
            </div>
          </div>

          {d.uv != null && (
            <div
              style={{
                ...TYPE.label,
                width: 150,
                padding: "6px 0",
                textAlign: "center",
                boxSizing: "border-box",
                color: d.uv > HIGH_UV ? PAPER : INK,
                backgroundColor: d.uv > HIGH_UV ? RED : PAPER,
                border: `3px solid ${INK}`,
              }}
            >
              UV {d.uv.toFixed(1)}
            </div>
          )}

          <div style={{ ...TYPE.title, width: 210, textAlign: "right" }}>
            {d.max}°<span style={{ fontSize: 30, fontWeight: 600 }}> / {d.min}°</span>
          </div>
        </div>
      ))}
    </section>
  );
}

function formatForecastDate(dateTime: string) {
  const d = new Date(dateTime);
  if (Number.isNaN(d.getTime())) return dateTime;

  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const compare = new Date(d);
  compare.setHours(0, 0, 0, 0);

  const diffDays = Math.round((compare.getTime() - today.getTime()) / 86_400_000);

  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Tomorrow";

  return d.toLocaleDateString([], { weekday: "long" });
}
