// A flat, client-side view of every entity, keyed by kind + id. The `entities`
// query seeds it and `events` subscription updates merge in by matching id.

export type EntityKind = "light" | "door" | "presence" | "environment";

export interface Entity {
  key: string;
  kind: EntityKind;
  id: string;
  name: string;
  capabilities?: readonly string[];
  on?: boolean | null;
  open?: boolean | null;
  present?: boolean | null;
  temperature?: number | null;
  humidity?: number | null;
  pressure?: number | null;
  lux?: number | null;
  uvIndex?: number | null;
  time?: string | null;
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
