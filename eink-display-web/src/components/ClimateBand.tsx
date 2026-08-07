import { COLUMN_W, HAIRLINE, ROW, TYPE } from "../theme";

type Reading = {
  readonly temperature: number | null | undefined;
  readonly humidity: number | null | undefined;
};

function Climate({
  label,
  reading,
  align,
}: {
  label: string;
  reading: Reading | null | undefined;
  align: "left" | "right";
}) {
  return (
    <div style={{ width: COLUMN_W, textAlign: align }}>
      <div style={TYPE.label}>{label}</div>
      <div
        style={{
          display: "flex",
          justifyContent: align === "left" ? "flex-start" : "flex-end",
          alignItems: "baseline",
          gap: 20,
          marginTop: 10,
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
  indoor,
}: {
  outdoor: Reading | null | undefined;
  indoor: Reading | null | undefined;
}) {
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
      <Climate label="Outside" reading={outdoor} align="left" />
      <Climate label="Living Room" reading={indoor} align="right" />
    </section>
  );
}
