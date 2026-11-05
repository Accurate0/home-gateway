/**
 * @generated SignedSource<<e0d5f54853a3c14ac19942da7d44dcce>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeAirPurifierMode = "AUTO" | "MANUAL" | "SLEEP" | "%future added value";
export type HomeCapability = "BRIGHTNESS" | "COLOUR_TEMP" | "HUMIDITY" | "LUX" | "PM_25" | "PRESSURE" | "RGB" | "TEMPERATURE" | "UV_INDEX" | "VOC_INDEX" | "%future added value";
export type HomeDeviceLockSource = "MANUAL" | "PHYSICAL" | "%future added value";
export type HomeEinkDisplayKind = "EINK_DISPLAY_FIRMWARE" | "TRMNL" | "%future added value";
export type HomeEinkMode = "DASHBOARD" | "IMMICH" | "REDDIT" | "%future added value";
export type HomeEntityCategory = "DISPLAYS" | "DOORS" | "ENVIRONMENT" | "LIGHTS" | "MEDIA" | "PLANTS" | "PRESENCE" | "VACUUMS" | "%future added value";
export type HomeGarageDoorState = "CLOSED" | "CLOSING" | "OPEN" | "OPENING" | "%future added value";
export type HomeOrientation = "LANDSCAPE" | "PORTRAIT" | "%future added value";
export type HomeRobotVacuumKind = "ROBOROCK" | "VALETUDO" | "%future added value";
export type DashboardEntitiesQuery$variables = Record<PropertyKey, never>;
export type DashboardEntitiesQuery$data = {
  readonly homeEntities: ReadonlyArray<{
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
    readonly capabilities?: ReadonlyArray<HomeCapability>;
    readonly category?: HomeEntityCategory;
    readonly cleanCount?: number | null | undefined;
    readonly config?: {
      readonly immichAlbum: string | null | undefined;
      readonly mode: HomeEinkMode;
      readonly orientation: HomeOrientation;
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
    readonly einkKind?: HomeEinkDisplayKind;
    readonly episode?: number | null | undefined;
    readonly fanSpeed?: string | null | undefined;
    readonly filterLife?: number | null | undefined;
    readonly garageState?: HomeGarageDoorState | null | undefined;
    readonly humidity?: number | null | undefined;
    readonly id?: string;
    readonly kind?: HomeRobotVacuumKind;
    readonly lastSeen?: any | null | undefined;
    readonly lock?: {
      readonly expiresAt: any | null | undefined;
      readonly source: HomeDeviceLockSource;
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
    readonly purifierMode?: HomeAirPurifierMode | null | undefined;
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
  readonly homeEntitySections: ReadonlyArray<{
    readonly category: HomeEntityCategory;
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
    "concreteType": "HomeEntitySection",
    "kind": "LinkedField",
    "name": "homeEntitySections",
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
    "name": "homeEntities",
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
        "type": "HomeLightEntity",
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
        "type": "HomeDoorEntity",
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
        "type": "HomePresenceEntity",
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
        "type": "HomeEnvironmentEntity",
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
        "type": "HomePlantEntity",
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
            "concreteType": "HomeEinkDisplayConfig",
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
                "name": "immichAlbum",
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
        "type": "HomeEinkDisplayEntity",
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
        "type": "HomeMediaPlayerEntity",
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
            "concreteType": "HomeDeviceBattery",
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
        "type": "HomeGarageDoorEntity",
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
        "type": "HomeAirPurifierEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "HomeDeviceLock",
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
        "type": "HomeLockableEntity",
        "abstractKey": "__isHomeLockableEntity"
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
        "type": "HomeRobotVacuumEntity",
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
    "type": "Query",
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
    "cacheID": "6b8d8278ee5934e96001b058bfc214da",
    "id": null,
    "metadata": {},
    "name": "DashboardEntitiesQuery",
    "operationKind": "query",
    "text": "query DashboardEntitiesQuery {\n  homeEntitySections {\n    category\n    title\n  }\n  homeEntities {\n    __typename\n    ... on HomeLightEntity {\n      category\n      id\n      name\n      room\n      capabilities\n      on\n      lastSeen\n    }\n    ... on HomeDoorEntity {\n      category\n      id\n      name\n      room\n      open\n      lastSeen\n    }\n    ... on HomePresenceEntity {\n      category\n      id\n      name\n      room\n      present\n      lastSeen\n    }\n    ... on HomeEnvironmentEntity {\n      category\n      id\n      name\n      room\n      capabilities\n      temperature\n      humidity\n      pressure\n      pm25\n      vocIndex\n      lux\n      uvIndex\n      time\n      lastSeen\n    }\n    ... on HomePlantEntity {\n      category\n      id\n      name\n      room\n      soilMoisture\n      time\n      lastSeen\n    }\n    ... on HomeEinkDisplayEntity {\n      category\n      id\n      name\n      einkKind: kind\n      room\n      batteryVoltage\n      batteryPercentage\n      lastSeen\n      config {\n        mode\n        view\n        immichAlbum\n        orientation\n        refresh\n        settle\n        sleepStart\n        sleepEnd\n      }\n    }\n    ... on HomeMediaPlayerEntity {\n      category\n      id\n      name\n      room\n      state\n      appName\n      source\n      mediaTitle\n      mediaArtist\n      mediaSeriesTitle\n      season\n      episode\n      positionSeconds\n      durationSeconds\n      progress\n      volumeLevel\n      muted\n      artworkUrl\n      lastSeen\n    }\n    ... on HomeGarageDoorEntity {\n      category\n      id\n      name\n      room\n      garageState: state\n      battery {\n        percentage\n      }\n      lastSeen\n    }\n    ... on HomeAirPurifierEntity {\n      category\n      id\n      name\n      room\n      on\n      purifierMode: mode\n      purifierSpeed: speed\n      purifierPm25: pm25\n      aqi\n      cadr\n      filterLife\n      displayOn: display\n      lastSeen\n    }\n    ... on HomeLockableEntity {\n      __isHomeLockableEntity: __typename\n      lock {\n        source\n        expiresAt\n      }\n    }\n    ... on HomeRobotVacuumEntity {\n      category\n      id\n      name\n      room\n      capabilities\n      kind\n      status\n      batteryPercentage\n      currentRoom\n      fanSpeed\n      currentCleanArea\n      cleanCount\n      lastSeen\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "994ac596e4584791f8d1c1829b7eea3c";

export default node;
