import { useEffect, useMemo, useState } from "react";
import { graphql, useLazyLoadQuery, useRelayEnvironment } from "react-relay";
import { requestSubscription } from "relay-runtime";
import type { AppEntitiesQuery } from "./__generated__/AppEntitiesQuery.graphql";
import type { AppEventsSubscription } from "./__generated__/AppEventsSubscription.graphql";
import EntityCard from "./components/EntityCard";
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
        on
      }
      ... on DoorEntity {
        id
        name
        open
      }
      ... on PresenceEntity {
        id
        name
        present
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

          const merged: Entity =
            "readings" in update && update.readings
              ? applyReadings({ ...existing }, update.readings)
              : { ...existing, ...update, kind, key };

          const next = new Map(prev);
          next.set(key, merged);
          return next;
        });
      },
    });
    return () => sub.dispose();
  }, [environment]);

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
    <div className="mx-auto max-w-6xl px-6 py-8">
      <header className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight">Home Gateway</h1>
        <p className="text-muted-foreground text-sm">
          {entities.size} entities · live
        </p>
      </header>

      {SECTIONS.map(({ kind, title }) => {
        const list = byKind.get(kind);
        if (!list || list.length === 0) return null;
        return (
          <section key={kind} className="mb-8">
            <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
              {title}
            </h2>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {list.map((entity) => (
                <EntityCard key={entity.key} entity={entity} />
              ))}
            </div>
          </section>
        );
      })}
    </div>
  );
}
