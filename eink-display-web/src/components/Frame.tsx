import type { ReactNode } from "react";
import { CONTENT_H, FONT, INK, PAD_X, PAD_Y, PAPER } from "../theme";

export default function Frame({ children }: { children: ReactNode }) {
  return (
    <div
      style={{
        width: "100%",
        height: "100vh",
        backgroundColor: PAPER,
        color: INK,
        fontFamily: FONT,
        padding: `${PAD_Y}px ${PAD_X}px`,
        boxSizing: "border-box",
        overflow: "hidden",
      }}
    >
      <div style={{ height: CONTENT_H }}>{children}</div>
    </div>
  );
}
