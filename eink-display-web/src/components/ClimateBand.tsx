import { graphql, useFragment } from "react-relay";
import type { ClimateBand_weather$key } from "./__generated__/ClimateBand_weather.graphql";
import WeatherIcon from "./WeatherIcon";
import { fromToday } from "../lib/time";
import { COLUMN_W, HAIRLINE, ROW, TYPE, uvInk } from "../theme";

const TodayFragment = graphql`
  fragment ClimateBand_weather on WeatherObject {
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

type Reading = {
  readonly temperature: number | null | undefined;
  readonly humidity: number | null | undefined;
};

function Outside({ reading }: { reading: Reading | null | undefined }) {
  return (
    <div style={{ width: COLUMN_W }}>
      <div
        style={{
          display: "flex",
          alignItems: "baseline",
          gap: 20,
        }}
      >
        <div style={{ fontSize: 84, fontWeight: 900, lineHeight: 1 }}>
          {reading?.temperature?.toFixed(1) ?? "--"}°
        </div>
        <div style={TYPE.body}>{reading?.humidity?.toFixed(0) ?? "--"}% RH</div>
      </div>
    </div>
  );
}

export default function ClimateBand({
  outdoor,
  weatherRef,
}: {
  outdoor: Reading | null | undefined;
  weatherRef: ClimateBand_weather$key | null | undefined;
}) {
  const data = useFragment(TodayFragment, weatherRef ?? null);
  const today = fromToday(data?.forecast?.days ?? [])[0];

  return (
    <section
      style={{
        height: ROW.climate,
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        borderBottom: HAIRLINE,
        boxSizing: "border-box",
      }}
    >
      <Outside reading={outdoor} />

      {today && (
        <div
          style={{
            width: COLUMN_W,
            display: "flex",
            alignItems: "center",
            justifyContent: "flex-end",
            gap: 24,
          }}
        >
          <div style={{ textAlign: "right" }}>
            <div style={TYPE.body}>{today.description}</div>
            {today.uv != null && (
              <div style={{ ...TYPE.body, marginTop: 4, color: uvInk(today.uv) }}>
                UV {today.uv.toFixed(1)}
              </div>
            )}
          </div>

          <div
            style={{
              fontSize: 84,
              fontWeight: 900,
              lineHeight: 1,
              whiteSpace: "nowrap",
            }}
          >
            {today.max}°<span style={TYPE.body}> / {today.min}°</span>
          </div>

          <WeatherIcon code={today.code} size={96} />
        </div>
      )}
    </section>
  );
}
