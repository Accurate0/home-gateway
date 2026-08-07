import { COLUMN_W, INK, PAPER, RED, ROW, RULE, TYPE } from "../theme";

export default function Header({
  updatedAt,
  stale,
}: {
  updatedAt: string;
  stale: boolean;
}) {
  return (
    <header
      style={{
        height: ROW.header,
        display: "flex",
        alignItems: "flex-end",
        justifyContent: "space-between",
        borderBottom: RULE,
        boxSizing: "border-box",
        paddingBottom: 16,
      }}
    >
      <div style={{ width: COLUMN_W }}>
        <div style={{ ...TYPE.display, textTransform: "uppercase" }}>
          {new Date().toLocaleDateString([], { weekday: "long" })}
        </div>
        <div style={{ ...TYPE.title, marginTop: 8 }}>
          {new Date().toLocaleDateString([], {
            day: "numeric",
            month: "long",
            year: "numeric",
          })}
        </div>
      </div>

      <div
        style={{
          width: COLUMN_W,
          textAlign: "right",
        }}
      >
        <div style={{ ...TYPE.label }}>Updated</div>
        <div
          style={{
            ...TYPE.title,
            marginTop: 8,
            display: "inline-block",
            padding: stale ? "6px 14px" : 0,
            backgroundColor: stale ? RED : "transparent",
            color: stale ? PAPER : INK,
          }}
        >
          {updatedAt}
        </div>
      </div>
    </header>
  );
}
