import { useEffect, useMemo, useState } from "react";
import {
  graphql,
  useLazyLoadQuery,
  useMutation,
  useRelayEnvironment,
} from "react-relay";
import { requestSubscription } from "relay-runtime";
import type { AppEntitiesQuery } from "./__generated__/AppEntitiesQuery.graphql";
import type { AppEventsSubscription } from "./__generated__/AppEventsSubscription.graphql";
import type { AppToggleLightMutation } from "./__generated__/AppToggleLightMutation.graphql";
import type { AppSetBrightnessMutation } from "./__generated__/AppSetBrightnessMutation.graphql";
import type { AppColourMoveMutation } from "./__generated__/AppColourMoveMutation.graphql";
import type { AppSetColourMutation } from "./__generated__/AppSetColourMutation.graphql";
import EntityCard, { type LightActions } from "./components/EntityCard";
import {
  applyReadings,
  entityKey,
  kindOf,
  type Entity,
  type EntityKind,
} from "./entities";

const EntitiesQuery = graphql`
  query AppEntitiesQuery {
    entities {
      __typename
      ... on LightEntity {
        id
        name
        capabilities
        on
        lastSeen
      }
      ... on DoorEntity {
        id
        name
        open
        lastSeen
      }
      ... on PresenceEntity {
        id
        name
        present
        lastSeen
      }
      ... on EnvironmentEntity {
        id
        name
        temperature
        humidity
        pressure
        lux
        uvIndex
        time
        lastSeen
      }
    }
  }
`;

const EventsSubscription = graphql`
  subscription AppEventsSubscription {
    events(filter: "*") {
      __typename
      ... on LightUpdate {
        id
        name
        on
      }
      ... on DoorUpdate {
        id
        name
        open
      }
      ... on PresenceUpdate {
        id
        name
        present
      }
      ... on EnvironmentUpdate {
        id
        name
        readings {
          metric
          value
        }
      }
    }
  }
`;

const ToggleLightMutation = graphql`
  mutation AppToggleLightMutation($id: String!) {
    light(id: $id) {
      toggle
    }
  }
`;

const SetBrightnessMutation = graphql`
  mutation AppSetBrightnessMutation($id: String!, $value: Int!) {
    light(id: $id) {
      setBrightness(input: { value: $value })
    }
  }
`;

const ColourMoveMutation = graphql`
  mutation AppColourMoveMutation($id: String!, $value: Int!) {
    light(id: $id) {
      colourTemperatureMove(input: { value: $value })
    }
  }
`;

const SetColourMutation = graphql`
  mutation AppSetColourMutation($id: String!, $hex: String!) {
    light(id: $id) {
      setColour(input: { hex: $hex })
    }
  }
`;

const SECTIONS: { kind: EntityKind; title: string }[] = [
  { kind: "light", title: "Lights" },
  { kind: "door", title: "Doors" },
  { kind: "presence", title: "Presence" },
  { kind: "environment", title: "Environment" },
];

function seedEntities(data: AppEntitiesQuery["response"]): Map<string, Entity> {
  const map = new Map<string, Entity>();
  for (const e of data.entities) {
    const kind = kindOf(e.__typename);
    if (!kind || !("id" in e)) continue;
    map.set(entityKey(kind, e.id), { ...e, kind, key: entityKey(kind, e.id) });
  }
  return map;
}

export default function App() {
  const data = useLazyLoadQuery<AppEntitiesQuery>(EntitiesQuery, {});
  const environment = useRelayEnvironment();
  const [entities, setEntities] = useState<Map<string, Entity>>(() =>
    seedEntities(data),
  );
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 30_000);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    const sub = requestSubscription<AppEventsSubscription>(environment, {
      subscription: EventsSubscription,
      variables: {},
      onNext: (response) => {
        const update = response?.events;
        const kind = kindOf(update?.__typename);
        if (!update || !kind || !("id" in update)) return;

        setEntities((prev) => {
          const key = entityKey(kind, update.id);
          const existing =
            prev.get(key) ??
            ({ key, kind, id: update.id, name: update.name } as Entity);

          const lastSeen = new Date().toISOString();
          const merged: Entity =
            "readings" in update && update.readings
              ? applyReadings({ ...existing, lastSeen }, update.readings)
              : { ...existing, ...update, kind, key, lastSeen };

          const next = new Map(prev);
          next.set(key, merged);
          return next;
        });
      },
    });
    return () => sub.dispose();
  }, [environment]);

  const [commitToggle] = useMutation<AppToggleLightMutation>(
    ToggleLightMutation,
  );
  const [commitBrightness] = useMutation<AppSetBrightnessMutation>(
    SetBrightnessMutation,
  );
  const [commitColourMove] = useMutation<AppColourMoveMutation>(
    ColourMoveMutation,
  );
  const [commitColour] = useMutation<AppSetColourMutation>(SetColourMutation);

  const flip = (key: string) =>
    setEntities((prev) => {
      const existing = prev.get(key);
      if (!existing || existing.kind !== "light") return prev;
      const next = new Map(prev);
      next.set(key, { ...existing, on: !existing.on });
      return next;
    });

  const lightActionsFor = (entity: Entity): LightActions => ({
    onToggle: () => {
      flip(entity.key);
      commitToggle({
        variables: { id: entity.id },
        onError: () => flip(entity.key),
      });
    },
    onSetBrightness: (value) =>
      commitBrightness({ variables: { id: entity.id, value } }),
    onColourMove: (value) =>
      commitColourMove({ variables: { id: entity.id, value } }),
    onSetColour: (hex) => commitColour({ variables: { id: entity.id, hex } }),
    canSetColour: entity.capabilities?.includes("RGB") ?? false,
  });

  const byKind = useMemo(() => {
    const groups = new Map<EntityKind, Entity[]>();
    for (const e of entities.values()) {
      const list = groups.get(e.kind) ?? [];
      list.push(e);
      groups.set(e.kind, list);
    }
    for (const list of groups.values()) {
      list.sort((a, b) => a.name.localeCompare(b.name));
    }
    return groups;
  }, [entities]);

  return (
    <div className="mx-auto max-w-6xl px-6 py-12">
      <header className="mb-10">
        <h1 className="font-display text-4xl font-semibold tracking-tight sm:text-5xl">
          Home Gateway
        </h1>
        <p className="text-muted-foreground mt-2 flex items-center gap-2 text-sm">
          <span className="bg-state-present relative flex size-2 rounded-full">
            <span className="bg-state-present absolute inline-flex size-full animate-ping rounded-full opacity-75" />
          </span>
          {entities.size} entities · live
        </p>
      </header>

      {SECTIONS.map(({ kind, title }) => {
        const list = byKind.get(kind);
        if (!list || list.length === 0) return null;
        return (
          <section key={kind} className="mb-10">
            <h2 className="text-muted-foreground mb-3 text-xs font-semibold tracking-widest uppercase">
              {title}
            </h2>
            <div className="grid auto-rows-[1fr] grid-flow-row-dense grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
              {list.map((entity) => (
                <EntityCard
                  key={entity.key}
                  entity={entity}
                  now={now}
                  lightActions={
                    entity.kind === "light"
                      ? lightActionsFor(entity)
                      : undefined
                  }
                />
              ))}
            </div>
          </section>
        );
      })}
    </div>
  );
}
