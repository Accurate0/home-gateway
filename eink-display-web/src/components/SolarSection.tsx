import SolarChart from "./SolarChart";
import Stat from "./Stat";
import type { SolarChart_solar$key } from "./__generated__/SolarChart_solar.graphql";
import {
  CHART_H,
  COLUMN_W,
  CONTENT_W,
  HAIRLINE,
  ROW,
  SOLAR_GAP,
  SOLAR_STATS_H,
  TYPE,
  uvInk,
} from "../theme";

export default function SolarSection({
  solarRef,
  last15Mins,
  last1Hour,
  uvLevel,
  todayProductionKwh,
}: {
  solarRef: SolarChart_solar$key;
  last15Mins: number | null | undefined;
  last1Hour: number | null | undefined;
  uvLevel: number | null | undefined;
  todayProductionKwh: number | null | undefined;
}) {
  return (
    <section
      style={{
        height: ROW.solar,
        paddingBottom: SOLAR_GAP,
        borderBottom: HAIRLINE,
        boxSizing: "border-box",
      }}
    >
      <div
        style={{
          height: SOLAR_STATS_H,
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <div
          style={{
            width: COLUMN_W,
            display: "flex",
            justifyContent: "space-between",
            paddingRight: 40,
            boxSizing: "border-box",
          }}
        >
          <Stat label="15m avg" value={last15Mins?.toFixed(0) ?? "--"} suffix="W" />
          <Stat label="1h avg" value={last1Hour?.toFixed(0) ?? "--"} suffix="W" />
          <Stat label="UV" value={uvLevel?.toFixed(1) ?? "--"} color={uvInk(uvLevel)} />
        </div>

        <div
          style={{
            width: COLUMN_W,
            textAlign: "right",
          }}
        >
          <div style={TYPE.label}>Today</div>
          <div style={{ fontSize: 64, fontWeight: 900, lineHeight: 1, marginTop: 6 }}>
            {todayProductionKwh?.toFixed(1) ?? "--"}
            <span style={{ fontSize: 32, fontWeight: 800 }}> kWh</span>
          </div>
        </div>
      </div>

      <SolarChart solarRef={solarRef} width={CONTENT_W} height={CHART_H} />
    </section>
  );
}
