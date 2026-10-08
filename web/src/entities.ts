import {
  differenceInDays,
  differenceInWeeks,
  formatDistanceStrict,
  type Locale,
} from "date-fns";
import { enUS } from "date-fns/locale/en-US";

// A flat, client-side view of every entity, keyed by kind + id. The `entities`
// query seeds it and `events` subscription updates merge in by matching id.

export type EntityKind =
  | "light"
  | "door"
  | "presence"
  | "environment"
  | "plant"
  | "einkDisplay"
  | "robotVacuum"
  | "mediaPlayer"
  | "garageDoor"
  | "airPurifier";

export type AirPurifierMode = "MANUAL" | "SLEEP" | "AUTO";

export type VacuumKind = "ROBOROCK" | "VALETUDO";

export type EinkKind = "TRMNL" | "EINK_DISPLAY_FIRMWARE";

export interface EinkDisplayConfig {
  mode: string;
  view?: string | null;
  album?: string | null;
  orientation: string;
  refresh?: string | null;
  settle?: string | null;
  sleepStart?: string | null;
  sleepEnd?: string | null;
}

export interface EinkDeviceConfig {
  refreshIntervalMins?: number | null;
  imageUrl?: string | null;
  clearScreen?: boolean | null;
}

export interface EinkStatus {
  nextWakeAt?: string | null;
  partialRefreshCount?: number | null;
  targetFirmwareVersion?: string | null;
  isCharging?: boolean | null;
  grace?: string | null;
  lead?: string | null;
  rtcReportedAt?: string | null;
  rtcReportedOffsetSecs?: number | null;
  rtcSyncedAt?: string | null;
  rtcDriftSecs?: number | null;
  rtcSyncDueAt?: string | null;
  lastWakeTraceUrl?: string | null;
}

export interface Entity {
  key: string;
  kind: EntityKind;
  category?: string | null;
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
  pm25?: number | null;
  vocIndex?: number | null;
  lux?: number | null;
  uvIndex?: number | null;
  soilMoisture?: number | null;
  time?: string | null;
  lastSeen?: string | null;
  batteryVoltage?: number | null;
  batteryPercentage?: number | null;
  config?: EinkDisplayConfig | null;
  deviceConfig?: EinkDeviceConfig | null;
  einkStatus?: EinkStatus | null;
  vacuumKind?: VacuumKind | null;
  einkKind?: EinkKind | null;
  status?: string | null;
  currentRoom?: string | null;
  fanSpeed?: string | null;
  currentCleanArea?: number | null;
  cleanCount?: number | null;
  state?: string | null;
  appName?: string | null;
  source?: string | null;
  mediaTitle?: string | null;
  mediaArtist?: string | null;
  mediaSeriesTitle?: string | null;
  season?: number | null;
  episode?: number | null;
  positionSeconds?: number | null;
  durationSeconds?: number | null;
  progress?: number | null;
  volumeLevel?: number | null;
  muted?: boolean | null;
  artworkUrl?: string | null;
  battery?: { readonly percentage?: number | null } | null;
  garageState?: string | null;
  purifierMode?: string | null;
  purifierSpeed?: number | null;
  purifierPm25?: number | null;
  aqi?: number | null;
  cadr?: number | null;
  filterLife?: number | null;
  displayOn?: boolean | null;
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
  PlantEntity: "plant",
  EinkDisplayEntity: "einkDisplay",
  RobotVacuumEntity: "robotVacuum",
  MediaPlayerEntity: "mediaPlayer",
  GarageDoorEntity: "garageDoor",
  AirPurifierEntity: "airPurifier",
  LightUpdate: "light",
  DoorUpdate: "door",
  PresenceUpdate: "presence",
  EnvironmentUpdate: "environment",
  PlantUpdate: "plant",
  DeviceBatteryUpdate: "einkDisplay",
  MediaPlayerUpdate: "mediaPlayer",
  GarageDoorUpdate: "garageDoor",
  AirPurifierUpdate: "airPurifier",
};

export function isGarageDoorMoving(state: string | null | undefined): boolean {
  return state === "OPENING" || state === "CLOSING";
}

export function kindOf(typename: string | undefined): EntityKind | null {
  return (typename && TYPENAME_TO_KIND[typename]) || null;
}

// Home Assistant media_player states that mean something is loaded and running.
// Derived on the client so an `events` update flips the tile without a refetch.
export function isPlaying(state: string | null | undefined): boolean {
  return state === "playing" || state === "buffering";
}

export function entityKey(kind: EntityKind, id: string): string {
  return `${kind}:${id}`;
}

// Metric names emitted by EnvironmentUpdate.readings mapped onto Entity fields.
const METRIC_FIELDS: Record<string, keyof Entity> = {
  temperature: "temperature",
  humidity: "humidity",
  pressure: "pressure",
  pm25: "pm25",
  voc_index: "vocIndex",
  lux: "lux",
  uv_index: "uvIndex",
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
