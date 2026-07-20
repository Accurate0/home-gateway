import {
  differenceInDays,
  differenceInWeeks,
  formatDistanceStrict,
  type Locale,
} from "date-fns";
import { enUS } from "date-fns/locale/en-US";

// A flat, client-side view of every entity, keyed by kind + id. The `entities`
// query seeds it and `events` subscription updates merge in by matching id.

export type EntityKind = "light" | "door" | "presence" | "environment";

export interface Entity {
  key: string;
  kind: EntityKind;
  id: string;
  name: string;
  capabilities?: readonly string[];
  room?: string | null;
  on?: boolean | null;
  open?: boolean | null;
  present?: boolean | null;
  temperature?: number | null;
  humidity?: number | null;
  pressure?: number | null;
  lux?: number | null;
  uvIndex?: number | null;
  time?: string | null;
  lastSeen?: string | null;
}

const SHORT_UNITS: Record<string, string> = {
  xSeconds: "s",
  xMinutes: "m",
  xHours: "h",
  xDays: "d",
  xMonths: "mo",
  xYears: "y",
};

const shortLocale: Locale = {
  ...enUS,
  formatDistance: (token, count) => {
    const unit = SHORT_UNITS[token];
    return unit ? `${count}${unit}` : enUS.formatDistance(token, count);
  },
};

export function formatLastSeen(
  lastSeen: string | null | undefined,
  now: number = Date.now(),
): string | null {
  if (lastSeen == null) return null;
  const then = Date.parse(lastSeen);
  if (Number.isNaN(then)) return null;
  if (now - then < 60_000) return "now";
  // date-fns jumps day -> month; surface weeks for the 1-4 week band.
  const weeks = differenceInWeeks(now, then);
  if (differenceInDays(now, then) >= 7 && weeks <= 4) return `${weeks}w`;
  return formatDistanceStrict(then, now, {
    roundingMethod: "floor",
    locale: shortLocale,
  });
}

const TYPENAME_TO_KIND: Record<string, EntityKind> = {
  LightEntity: "light",
  DoorEntity: "door",
  PresenceEntity: "presence",
  EnvironmentEntity: "environment",
  LightUpdate: "light",
  DoorUpdate: "door",
  PresenceUpdate: "presence",
  EnvironmentUpdate: "environment",
};

export function kindOf(typename: string | undefined): EntityKind | null {
  return (typename && TYPENAME_TO_KIND[typename]) || null;
}

export function entityKey(kind: EntityKind, id: string): string {
  return `${kind}:${id}`;
}

// Metric names emitted by EnvironmentUpdate.readings mapped onto Entity fields.
const METRIC_FIELDS: Record<string, keyof Entity> = {
  temperature: "temperature",
  humidity: "humidity",
  pressure: "pressure",
  lux: "lux",
  uvIndex: "uvIndex",
};

export function applyReadings(
  entity: Entity,
  readings: readonly { readonly metric: string; readonly value: number }[],
): Entity {
  const next = { ...entity };
  for (const { metric, value } of readings) {
    const field = METRIC_FIELDS[metric];
    if (field) (next[field] as number) = value;
  }
  return next;
}
