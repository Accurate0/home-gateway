import { INK, TYPE } from "../theme";

export default function Stat({
  label,
  value,
  suffix,
  color = INK,
  align = "left",
  size = TYPE.title.fontSize,
}: {
  label: string;
  value: string;
  suffix?: string;
  color?: string;
  align?: "left" | "right";
  size?: number;
}) {
  return (
    <div style={{ textAlign: align }}>
      <div style={{ ...TYPE.label, color: INK }}>{label}</div>
      <div
        style={{
          fontSize: size,
          fontWeight: 900,
          lineHeight: 1,
          color,
          marginTop: 6,
          whiteSpace: "nowrap",
        }}
      >
        {value}
        {suffix && (
          <span style={{ fontSize: Math.round(size * 0.5), fontWeight: 800 }}>{suffix}</span>
        )}
      </div>
    </div>
  );
}
