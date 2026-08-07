import { COLUMN_W, HAIRLINE, INK, LOW_BATTERY_PCT, PAPER, RED, ROW, TYPE } from "../theme";

type Panel = {
  readonly name: string;
  readonly batteryPercentage: number | null | undefined;
};

function Battery({ panel, align }: { panel: Panel | null | undefined; align: "left" | "right" }) {
  const pct = panel?.batteryPercentage;
  const low = pct != null && pct <= LOW_BATTERY_PCT;

  return (
    <div
      style={{
        width: COLUMN_W,
        display: "flex",
        flexDirection: align === "left" ? "row" : "row-reverse",
        alignItems: "center",
        gap: 20,
      }}
    >
      <div
        style={{
          ...TYPE.title,
          padding: "8px 18px",
          backgroundColor: low ? RED : PAPER,
          color: low ? PAPER : INK,
          border: `4px solid ${low ? RED : INK}`,
        }}
      >
        {pct?.toFixed(0) ?? "--"}%
      </div>
      <div style={TYPE.label}>{panel?.name ?? "Unknown panel"}</div>
    </div>
  );
}

export default function PanelBattery({
  hallway,
  livingRoom,
}: {
  hallway: Panel | null | undefined;
  livingRoom: Panel | null | undefined;
}) {
  return (
    <footer
      style={{
        height: ROW.footer,
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        borderTop: HAIRLINE,
        boxSizing: "border-box",
      }}
    >
      <Battery panel={hallway} align="left" />
      <Battery panel={livingRoom} align="right" />
    </footer>
  );
}
