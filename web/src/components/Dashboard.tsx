import { useEffect, useMemo, useState } from "react";
import {
  graphql,
  useLazyLoadQuery,
  useMutation,
  useRelayEnvironment,
} from "react-relay";
import { fetchQuery, requestSubscription } from "relay-runtime";
import type { DashboardEntitiesQuery } from "./__generated__/DashboardEntitiesQuery.graphql";
import type { DashboardEventsSubscription } from "./__generated__/DashboardEventsSubscription.graphql";
import type { DashboardSetOnMutation } from "./__generated__/DashboardSetOnMutation.graphql";
import type { DashboardSetOffMutation } from "./__generated__/DashboardSetOffMutation.graphql";
import type { DashboardSetBrightnessMutation } from "./__generated__/DashboardSetBrightnessMutation.graphql";
import type { DashboardColourMoveMutation } from "./__generated__/DashboardColourMoveMutation.graphql";
import type { DashboardSetColourMutation } from "./__generated__/DashboardSetColourMutation.graphql";
import type { DashboardVacuumStartMutation } from "./__generated__/DashboardVacuumStartMutation.graphql";
import type { DashboardVacuumStopMutation } from "./__generated__/DashboardVacuumStopMutation.graphql";
import type { DashboardVacuumDockMutation } from "./__generated__/DashboardVacuumDockMutation.graphql";
import type { DashboardEinkConfigQuery } from "./__generated__/DashboardEinkConfigQuery.graphql";
import type { DashboardTakeScreenshotMutation } from "./__generated__/DashboardTakeScreenshotMutation.graphql";
import EntityCard, {
  type EinkActions,
  type LightActions,
  type VacuumActions,
} from "./EntityCard";
import {
  applyReadings,
  entityKey,
  kindOf,
  type Entity,
} from "../entities";

const EntitiesQuery = graphql`
  query DashboardEntitiesQuery {
    entitySections {
      category
      title
    }
    entities {
      __typename
      ... on LightEntity {
        category
        id
        name
        room
        capabilities
        on
        lastSeen
      }
      ... on DoorEntity {
        category
        id
        name
        room
        open
        lastSeen
      }
      ... on PresenceEntity {
        category
        id
        name
        room
        present
        lastSeen
      }
      ... on EnvironmentEntity {
        category
        id
        name
        room
        capabilities
        temperature
        humidity
        pressure
        lux
        uvIndex
        time
        lastSeen
      }
      ... on EinkDisplayEntity {
        category
        id
        name
        einkKind: kind
        room
        batteryVoltage
        batteryPercentage
        lastSeen
        config {
          mode
          view
          album
          orientation
          refresh
          settle
          sleepStart
          sleepEnd
        }
      }
      ... on RobotVacuumEntity {
        category
        id
        name
        room
        capabilities
        kind
        status
        batteryPercentage
        currentRoom
        fanSpeed
        currentCleanArea
        cleanCount
        lastSeen
      }
    }
  }
`;

const EventsSubscription = graphql`
  subscription DashboardEventsSubscription {
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

const SetOnMutation = graphql`
  mutation DashboardSetOnMutation($id: String!) {
    light(id: $id) {
      on
    }
  }
`;

const SetOffMutation = graphql`
  mutation DashboardSetOffMutation($id: String!) {
    light(id: $id) {
      off
    }
  }
`;

const SetBrightnessMutation = graphql`
  mutation DashboardSetBrightnessMutation($id: String!, $value: Int!) {
    light(id: $id) {
      setBrightness(input: { value: $value })
    }
  }
`;

const ColourMoveMutation = graphql`
  mutation DashboardColourMoveMutation($id: String!, $value: Int!) {
    light(id: $id) {
      colourTemperatureMove(input: { value: $value })
    }
  }
`;

const SetColourMutation = graphql`
  mutation DashboardSetColourMutation($id: String!, $hex: String!) {
    light(id: $id) {
      setColour(input: { hex: $hex })
    }
  }
`;

const VacuumStartMutation = graphql`
  mutation DashboardVacuumStartMutation($id: String!) {
    robotVacuum(id: $id) {
      start
    }
  }
`;

const VacuumStopMutation = graphql`
  mutation DashboardVacuumStopMutation($id: String!) {
    robotVacuum(id: $id) {
      stop
    }
  }
`;

const VacuumDockMutation = graphql`
  mutation DashboardVacuumDockMutation($id: String!) {
    robotVacuum(id: $id) {
      dock
    }
  }
`;

const EinkConfigQuery = graphql`
  query DashboardEinkConfigQuery($id: String!) {
    einkDisplay(id: $id) {
      deviceConfig {
        refreshIntervalMins
        imageUrl
        clearScreen
      }
    }
  }
`;

const TakeScreenshotMutation = graphql`
  mutation DashboardTakeScreenshotMutation($id: String!) {
    einkDisplay(id: $id) {
      takeScreenshot
    }
  }
`;

type GroupBy = "type" | "room";

const GROUP_BY_KEY = "dashboard:groupBy";

const UNASSIGNED = "Unassigned";

function titleCase(slug: string): string {
  return slug
    .split(/[-_\s]+/)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

function groupByRoom(entities: Iterable<Entity>): [string, Entity[]][] {
  const groups = new Map<string, Entity[]>();
  for (const e of entities) {
    const title = e.room ? titleCase(e.room) : UNASSIGNED;
    const list = groups.get(title) ?? [];
    list.push(e);
    groups.set(title, list);
  }
  for (const list of groups.values()) {
    list.sort((a, b) => a.name.localeCompare(b.name));
  }
  return [...groups.entries()].sort(([a], [b]) => {
    if (a === UNASSIGNED) return 1;
    if (b === UNASSIGNED) return -1;
    return a.localeCompare(b);
  });
}

function seedEntities(
  data: DashboardEntitiesQuery["response"],
): Map<string, Entity> {
  const map = new Map<string, Entity>();
  for (const e of data.entities) {
    const kind = kindOf(e.__typename);
    if (!kind || !("id" in e)) continue;
    // RobotVacuumEntity exposes a `kind` (ROBOROCK/VALETUDO) that our EntityKind
    // `kind` shadows on spread, so lift it to `vacuumKind` before overwriting.
    const vacuumKind =
      "kind" in e && (e.kind === "ROBOROCK" || e.kind === "VALETUDO")
        ? e.kind
        : undefined;
    const einkKind =
      "einkKind" in e &&
      (e.einkKind === "TRMNL" || e.einkKind === "EINK_DISPLAY_FIRMWARE")
        ? e.einkKind
        : undefined;
    map.set(entityKey(kind, e.id), {
      ...e,
      kind,
      vacuumKind,
      einkKind,
      key: entityKey(kind, e.id),
    });
  }
  return map;
}

export default function Dashboard() {
  const data = useLazyLoadQuery<DashboardEntitiesQuery>(EntitiesQuery, {});
  const environment = useRelayEnvironment();
  const [entities, setEntities] = useState<Map<string, Entity>>(() =>
    seedEntities(data),
  );
  const [now, setNow] = useState(() => Date.now());
  const [groupBy, setGroupBy] = useState<GroupBy>(() =>
    localStorage.getItem(GROUP_BY_KEY) === "room" ? "room" : "type",
  );

  useEffect(() => {
    localStorage.setItem(GROUP_BY_KEY, groupBy);
  }, [groupBy]);

  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 30_000);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    const sub = requestSubscription<DashboardEventsSubscription>(environment, {
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

  const [commitOn] = useMutation<DashboardSetOnMutation>(SetOnMutation);
  const [commitOff] = useMutation<DashboardSetOffMutation>(SetOffMutation);
  const [commitBrightness] = useMutation<DashboardSetBrightnessMutation>(
    SetBrightnessMutation,
  );
  const [commitColourMove] = useMutation<DashboardColourMoveMutation>(
    ColourMoveMutation,
  );
  const [commitColour] =
    useMutation<DashboardSetColourMutation>(SetColourMutation);
  const [commitVacuumStart] =
    useMutation<DashboardVacuumStartMutation>(VacuumStartMutation);
  const [commitVacuumStop] =
    useMutation<DashboardVacuumStopMutation>(VacuumStopMutation);
  const [commitVacuumDock] =
    useMutation<DashboardVacuumDockMutation>(VacuumDockMutation);
  const [commitTakeScreenshot] = useMutation<DashboardTakeScreenshotMutation>(
    TakeScreenshotMutation,
  );

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
      const desiredOn = !entity.on;
      flip(entity.key);
      const commit = desiredOn ? commitOn : commitOff;
      commit({
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

  const vacuumActionsFor = (entity: Entity): VacuumActions => ({
    onStart: () => commitVacuumStart({ variables: { id: entity.id } }),
    onStop: () => commitVacuumStop({ variables: { id: entity.id } }),
    onDock: () => commitVacuumDock({ variables: { id: entity.id } }),
  });

  const einkActionsFor = (entity: Entity): EinkActions => ({
    onTakeScreenshot: () =>
      new Promise<void>((resolve, reject) => {
        commitTakeScreenshot({
          variables: { id: entity.id },
          onCompleted: () => resolve(),
          onError: (error) => reject(error),
        });
      }),
    onReload: () => {
      fetchQuery<DashboardEinkConfigQuery>(
        environment,
        EinkConfigQuery,
        { id: entity.id },
        { fetchPolicy: "network-only" },
      ).subscribe({
        next: (response) => {
          const deviceConfig = response.einkDisplay?.deviceConfig;
          if (!deviceConfig) return;
          setEntities((prev) => {
            const existing = prev.get(entity.key);
            if (!existing) return prev;
            const next = new Map(prev);
            next.set(entity.key, { ...existing, deviceConfig });
            return next;
          });
        },
      });
    },
  });

  const sections = useMemo(() => {
    if (groupBy === "room") return groupByRoom(entities.values());

    // The backend owns categorisation and section order; we only bucket entities
    // by the `category` it stamps on each one and render sections in that order.
    const groups = new Map<string, Entity[]>();
    for (const e of entities.values()) {
      const category = e.category ?? UNASSIGNED;
      const list = groups.get(category) ?? [];
      list.push(e);
      groups.set(category, list);
    }
    for (const list of groups.values()) {
      list.sort((a, b) => a.name.localeCompare(b.name));
    }

    const result: [string, Entity[]][] = [];
    for (const { category, title } of data.entitySections) {
      const list = groups.get(category);
      if (list && list.length > 0) result.push([title, list]);
    }
    const uncategorised = groups.get(UNASSIGNED);
    if (uncategorised && uncategorised.length > 0) {
      result.push([UNASSIGNED, uncategorised]);
    }
    return result;
  }, [entities, groupBy, data.entitySections]);

  return (
    <div>
      <div className="mb-8 flex items-center justify-between gap-4">
        <p className="text-muted-foreground flex items-center gap-2 text-sm">
          <span className="bg-state-present relative flex size-2 rounded-full">
            <span className="bg-state-present absolute inline-flex size-full animate-ping rounded-full opacity-75" />
          </span>
          {entities.size} entities · live
        </p>

        <div className="border-border flex rounded-md border text-xs">
          {(["type", "room"] as GroupBy[]).map((mode) => (
            <button
              key={mode}
              type="button"
              onClick={() => setGroupBy(mode)}
              aria-pressed={groupBy === mode}
              className={`px-3 py-1.5 capitalize first:rounded-l-md last:rounded-r-md ${
                groupBy === mode
                  ? "bg-muted text-foreground font-medium"
                  : "text-muted-foreground"
              }`}
            >
              {mode}
            </button>
          ))}
        </div>
      </div>

      {sections.map(([title, list]) => (
        <section key={title} className="mb-10">
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
                  vacuumActions={
                    entity.kind === "robotVacuum"
                      ? vacuumActionsFor(entity)
                      : undefined
                  }
                  einkActions={
                    entity.kind === "einkDisplay"
                      ? einkActionsFor(entity)
                      : undefined
                  }
                />
              ))}
            </div>
          </section>
      ))}
    </div>
  );
}
