export const INK = "#000000";
export const PAPER = "#ffffff";
export const YELLOW = "#ffff00";
export const RED = "#ff0000";
export const BLUE = "#0000ff";
export const GREEN = "#00ff00";

export const FONT = "'Inter Variable', Inter, system-ui, sans-serif";

export const PANEL = {
  portrait: { width: 1200, height: 1600 },
  landscape: { width: 1600, height: 1200 },
} as const;

export const PAD_X = 40;
export const PAD_Y = 24;

export const SPLIT = 600;
export const GUTTER = 8;

export const CONTENT_W = PANEL.portrait.width - PAD_X * 2;
export const CONTENT_H = PANEL.portrait.height - PAD_Y * 2;

export const COLUMN_W = SPLIT - PAD_X - GUTTER / 2;
export const RIGHT_COLUMN_X = SPLIT + GUTTER / 2;

export const ROW = {
  header: 148,
  climate: 156,
  solar: 608,
  forecast: 540,
  footer: 100,
} as const;

export const SOLAR_STATS_H = 170;
export const CHART_H = ROW.solar - SOLAR_STATS_H;

export const RULE = `4px solid ${INK}`;
export const HAIRLINE = `3px solid ${INK}`;

export const TYPE = {
  display: { fontSize: 76, fontWeight: 900, lineHeight: 1 },
  hero: { fontSize: 64, fontWeight: 900, lineHeight: 1 },
  title: { fontSize: 40, fontWeight: 800, lineHeight: 1.05 },
  body: { fontSize: 30, fontWeight: 700, lineHeight: 1.1 },
  label: { fontSize: 22, fontWeight: 800, letterSpacing: 1.5, textTransform: "uppercase" },
} as const;

export const LOW_BATTERY_PCT = 25;

export const HIGH_UV = 5;

export function uvInk(uv: number | null | undefined) {
  return uv != null && uv > HIGH_UV ? RED : INK;
}
