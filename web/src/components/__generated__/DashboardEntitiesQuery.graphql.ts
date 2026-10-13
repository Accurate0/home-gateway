/**
 * @generated SignedSource<<5d94538388856d186e8f09f5644f3ab2>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AirPurifierMode = "AUTO" | "MANUAL" | "SLEEP" | "%future added value";
export type Capability = "BRIGHTNESS" | "COLOUR_TEMP" | "HUMIDITY" | "LUX" | "PM_25" | "PRESSURE" | "RGB" | "TEMPERATURE" | "UV_INDEX" | "VOC_INDEX" | "%future added value";
export type DeviceLockSource = "MANUAL" | "PHYSICAL" | "%future added value";
export type EinkDisplayKind = "EINK_DISPLAY_FIRMWARE" | "TRMNL" | "%future added value";
export type EinkMode = "ALBUM" | "DASHBOARD" | "REDDIT" | "%future added value";
export type EntityCategory = "DISPLAYS" | "DOORS" | "ENVIRONMENT" | "LIGHTS" | "MEDIA" | "PLANTS" | "PRESENCE" | "VACUUMS" | "%future added value";
export type GarageDoorState = "CLOSED" | "CLOSING" | "OPEN" | "OPENING" | "%future added value";
export type Orientation = "LANDSCAPE" | "PORTRAIT" | "%future added value";
export type RobotVacuumKind = "ROBOROCK" | "VALETUDO" | "%future added value";
export type DashboardEntitiesQuery$variables = Record<PropertyKey, never>;
export type DashboardEntitiesQuery$data = {
  readonly entities: ReadonlyArray<{
    readonly __typename: string;
    readonly appName?: string | null | undefined;
    readonly aqi?: number | null | undefined;
    readonly artworkUrl?: string | null | undefined;
    readonly battery?: {
      readonly percentage: number | null | undefined;
    } | null | undefined;
    readonly batteryPercentage?: number | null | undefined;
    readonly batteryVoltage?: number | null | undefined;
    readonly cadr?: number | null | undefined;
    readonly capabilities?: ReadonlyArray<Capability>;
    readonly category?: EntityCategory;
    readonly cleanCount?: number | null | undefined;
    readonly config?: {
      readonly album: string | null | undefined;
      readonly mode: EinkMode;
      readonly orientation: Orientation;
      readonly refresh: string;
      readonly settle: string | null | undefined;
      readonly sleepEnd: string | null | undefined;
      readonly sleepStart: string | null | undefined;
      readonly view: string | null | undefined;
    } | null | undefined;
    readonly currentCleanArea?: number | null | undefined;
    readonly currentRoom?: string | null | undefined;
    readonly displayOn?: boolean | null | undefined;
    readonly durationSeconds?: number | null | undefined;
    readonly einkKind?: EinkDisplayKind;
    readonly episode?: number | null | undefined;
    readonly fanSpeed?: string | null | undefined;
    readonly filterLife?: number | null | undefined;
    readonly garageState?: GarageDoorState | null | undefined;
    readonly humidity?: number | null | undefined;
    readonly id?: string;
    readonly kind?: RobotVacuumKind;
    readonly lastSeen?: any | null | undefined;
    readonly lock?: {
      readonly expiresAt: any | null | undefined;
      readonly source: DeviceLockSource;
    } | null | undefined;
    readonly lux?: number | null | undefined;
    readonly mediaArtist?: string | null | undefined;
    readonly mediaSeriesTitle?: string | null | undefined;
    readonly mediaTitle?: string | null | undefined;
    readonly muted?: boolean | null | undefined;
    readonly name?: string;
    readonly on?: boolean | null | undefined;
    readonly open?: boolean | null | undefined;
    readonly pm25?: number | null | undefined;
    readonly positionSeconds?: number | null | undefined;
    readonly present?: boolean | null | undefined;
    readonly pressure?: number | null | undefined;
    readonly progress?: number | null | undefined;
    readonly purifierMode?: AirPurifierMode | null | undefined;
    readonly purifierPm25?: number | null | undefined;
    readonly purifierSpeed?: number | null | undefined;
    readonly room?: string | null | undefined;
    readonly season?: number | null | undefined;
    readonly soilMoisture?: number | null | undefined;
    readonly source?: string | null | undefined;
    readonly state?: string | null | undefined;
    readonly status?: string | null | undefined;
    readonly temperature?: number | null | undefined;
    readonly time?: any | null | undefined;
    readonly uvIndex?: number | null | undefined;
    readonly vocIndex?: number | null | undefined;
    readonly volumeLevel?: number | null | undefined;
  }>;
  readonly entitySections: ReadonlyArray<{
    readonly category: EntityCategory;
    readonly title: string;
  }>;
};
export type DashboardEntitiesQuery = {
  response: DashboardEntitiesQuery$data;
  variables: DashboardEntitiesQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "category",
  "storageKey": null
},
v1 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "room",
  "storageKey": null
},
v4 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "capabilities",
  "storageKey": null
},
v5 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "on",
  "storageKey": null
},
v6 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "lastSeen",
  "storageKey": null
},
v7 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "time",
  "storageKey": null
},
v8 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "batteryPercentage",
  "storageKey": null
},
v9 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "source",
  "storageKey": null
},
v10 = [
  {
    "alias": null,
    "args": null,
    "concreteType": "EntitySection",
    "kind": "LinkedField",
    "name": "entitySections",
    "plural": true,
    "selections": [
      (v0/*:: as any*/),
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "title",
        "storageKey": null
      }
    ],
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "concreteType": null,
    "kind": "LinkedField",
    "name": "entities",
    "plural": true,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "__typename",
        "storageKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          (v4/*:: as any*/),
          (v5/*:: as any*/),
          (v6/*:: as any*/)
        ],
        "type": "LightEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "open",
            "storageKey": null
          },
          (v6/*:: as any*/)
        ],
        "type": "DoorEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "present",
            "storageKey": null
          },
          (v6/*:: as any*/)
        ],
        "type": "PresenceEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          (v4/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "temperature",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "humidity",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "pressure",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "pm25",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "vocIndex",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "lux",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "uvIndex",
            "storageKey": null
          },
          (v7/*:: as any*/),
          (v6/*:: as any*/)
        ],
        "type": "EnvironmentEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "soilMoisture",
            "storageKey": null
          },
          (v7/*:: as any*/),
          (v6/*:: as any*/)
        ],
        "type": "PlantEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          {
            "alias": "einkKind",
            "args": null,
            "kind": "ScalarField",
            "name": "kind",
            "storageKey": null
          },
          (v3/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "batteryVoltage",
            "storageKey": null
          },
          (v8/*:: as any*/),
          (v6/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "concreteType": "EinkDisplayConfig",
            "kind": "LinkedField",
            "name": "config",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "mode",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "view",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "album",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "orientation",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "refresh",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "settle",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "sleepStart",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "sleepEnd",
                "storageKey": null
              }
            ],
            "storageKey": null
          }
        ],
        "type": "EinkDisplayEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "state",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "appName",
            "storageKey": null
          },
          (v9/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "mediaTitle",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "mediaArtist",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "mediaSeriesTitle",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "season",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "episode",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "positionSeconds",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "durationSeconds",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "progress",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "volumeLevel",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "muted",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "artworkUrl",
            "storageKey": null
          },
          (v6/*:: as any*/)
        ],
        "type": "MediaPlayerEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          {
            "alias": "garageState",
            "args": null,
            "kind": "ScalarField",
            "name": "state",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "concreteType": "DeviceBattery",
            "kind": "LinkedField",
            "name": "battery",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "percentage",
                "storageKey": null
              }
            ],
            "storageKey": null
          },
          (v6/*:: as any*/)
        ],
        "type": "GarageDoorEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          (v5/*:: as any*/),
          {
            "alias": "purifierMode",
            "args": null,
            "kind": "ScalarField",
            "name": "mode",
            "storageKey": null
          },
          {
            "alias": "purifierSpeed",
            "args": null,
            "kind": "ScalarField",
            "name": "speed",
            "storageKey": null
          },
          {
            "alias": "purifierPm25",
            "args": null,
            "kind": "ScalarField",
            "name": "pm25",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "aqi",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "cadr",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "filterLife",
            "storageKey": null
          },
          {
            "alias": "displayOn",
            "args": null,
            "kind": "ScalarField",
            "name": "display",
            "storageKey": null
          },
          (v6/*:: as any*/)
        ],
        "type": "AirPurifierEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "DeviceLock",
            "kind": "LinkedField",
            "name": "lock",
            "plural": false,
            "selections": [
              (v9/*:: as any*/),
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "expiresAt",
                "storageKey": null
              }
            ],
            "storageKey": null
          }
        ],
        "type": "LockableEntity",
        "abstractKey": "__isLockableEntity"
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          (v3/*:: as any*/),
          (v4/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "kind",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "status",
            "storageKey": null
          },
          (v8/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "currentRoom",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "fanSpeed",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "currentCleanArea",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "cleanCount",
            "storageKey": null
          },
          (v6/*:: as any*/)
        ],
        "type": "RobotVacuumEntity",
        "abstractKey": null
      }
    ],
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "DashboardEntitiesQuery",
    "selections": (v10/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "DashboardEntitiesQuery",
    "selections": (v10/*:: as any*/)
  },
  "params": {
    "cacheID": "fa35684bf45861ef3c77e339be2d612c",
    "id": null,
    "metadata": {},
    "name": "DashboardEntitiesQuery",
    "operationKind": "query",
    "text": "query DashboardEntitiesQuery {\n  entitySections {\n    category\n    title\n  }\n  entities {\n    __typename\n    ... on LightEntity {\n      category\n      id\n      name\n      room\n      capabilities\n      on\n      lastSeen\n    }\n    ... on DoorEntity {\n      category\n      id\n      name\n      room\n      open\n      lastSeen\n    }\n    ... on PresenceEntity {\n      category\n      id\n      name\n      room\n      present\n      lastSeen\n    }\n    ... on EnvironmentEntity {\n      category\n      id\n      name\n      room\n      capabilities\n      temperature\n      humidity\n      pressure\n      pm25\n      vocIndex\n      lux\n      uvIndex\n      time\n      lastSeen\n    }\n    ... on PlantEntity {\n      category\n      id\n      name\n      room\n      soilMoisture\n      time\n      lastSeen\n    }\n    ... on EinkDisplayEntity {\n      category\n      id\n      name\n      einkKind: kind\n      room\n      batteryVoltage\n      batteryPercentage\n      lastSeen\n      config {\n        mode\n        view\n        album\n        orientation\n        refresh\n        settle\n        sleepStart\n        sleepEnd\n      }\n    }\n    ... on MediaPlayerEntity {\n      category\n      id\n      name\n      room\n      state\n      appName\n      source\n      mediaTitle\n      mediaArtist\n      mediaSeriesTitle\n      season\n      episode\n      positionSeconds\n      durationSeconds\n      progress\n      volumeLevel\n      muted\n      artworkUrl\n      lastSeen\n    }\n    ... on GarageDoorEntity {\n      category\n      id\n      name\n      room\n      garageState: state\n      battery {\n        percentage\n      }\n      lastSeen\n    }\n    ... on AirPurifierEntity {\n      category\n      id\n      name\n      room\n      on\n      purifierMode: mode\n      purifierSpeed: speed\n      purifierPm25: pm25\n      aqi\n      cadr\n      filterLife\n      displayOn: display\n      lastSeen\n    }\n    ... on LockableEntity {\n      __isLockableEntity: __typename\n      lock {\n        source\n        expiresAt\n      }\n    }\n    ... on RobotVacuumEntity {\n      category\n      id\n      name\n      room\n      capabilities\n      kind\n      status\n      batteryPercentage\n      currentRoom\n      fanSpeed\n      currentCleanArea\n      cleanCount\n      lastSeen\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "a07e9b97b5a326e5f2c9fe3fbdf4c5a2";

export default node;
