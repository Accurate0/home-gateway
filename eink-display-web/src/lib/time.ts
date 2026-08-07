const TZ = "Australia/Perth";

const updatedFormatter = new Intl.DateTimeFormat("en-AU", {
  timeZone: TZ,
  weekday: "short",
  hour: "2-digit",
  minute: "2-digit",
  hour12: false,
});

export function formatUpdatedAt(date: Date) {
  return updatedFormatter.format(date);
}

export function perthMidnightISO() {
  const parts = new Intl.DateTimeFormat("en-AU", {
    timeZone: TZ,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(new Date());

  const part = (type: string) => parts.find((p) => p.type === type)?.value;

  return new Date(`${part("year")}-${part("month")}-${part("day")}T00:00:00+08:00`).toISOString();
}
