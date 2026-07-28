/**
 * @generated SignedSource<<3e31d917e2d4e66f043d530dbdf0f77c>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type Capability = "BRIGHTNESS" | "COLOUR_TEMP" | "HUMIDITY" | "LUX" | "PRESSURE" | "RGB" | "TEMPERATURE" | "UV_INDEX" | "%future added value";
export type EinkMode = "ALBUM" | "DASHBOARD" | "%future added value";
export type EntityCategory = "DISPLAYS" | "DOORS" | "ENVIRONMENT" | "LIGHTS" | "PRESENCE" | "VACUUMS" | "%future added value";
export type Orientation = "LANDSCAPE" | "PORTRAIT" | "%future added value";
export type RobotVacuumKind = "ROBOROCK" | "VALETUDO" | "%future added value";
export type DashboardEntitiesQuery$variables = Record<PropertyKey, never>;
export type DashboardEntitiesQuery$data = {
  readonly entities: ReadonlyArray<{
    readonly __typename: "DoorEntity";
    readonly category: EntityCategory;
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly open: boolean | null | undefined;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "EinkDisplayEntity";
    readonly batteryPercentage: number | null | undefined;
    readonly batteryVoltage: number | null | undefined;
    readonly category: EntityCategory;
    readonly config: {
      readonly album: string | null | undefined;
      readonly mode: EinkMode;
      readonly orientation: Orientation;
      readonly refresh: string | null | undefined;
      readonly settle: string | null | undefined;
      readonly sleepEnd: string | null | undefined;
      readonly sleepStart: string | null | undefined;
      readonly view: string | null | undefined;
    } | null | undefined;
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "EnvironmentEntity";
    readonly capabilities: ReadonlyArray<Capability>;
    readonly category: EntityCategory;
    readonly humidity: number | null | undefined;
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly lux: number | null | undefined;
    readonly name: string;
    readonly pressure: number | null | undefined;
    readonly room: string | null | undefined;
    readonly temperature: number | null | undefined;
    readonly time: any | null | undefined;
    readonly uvIndex: number | null | undefined;
  } | {
    readonly __typename: "LightEntity";
    readonly capabilities: ReadonlyArray<Capability>;
    readonly category: EntityCategory;
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly on: boolean | null | undefined;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "PresenceEntity";
    readonly category: EntityCategory;
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly present: boolean | null | undefined;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "RobotVacuumEntity";
    readonly batteryPercentage: number | null | undefined;
    readonly capabilities: ReadonlyArray<Capability>;
    readonly category: EntityCategory;
    readonly cleanCount: number | null | undefined;
    readonly currentCleanArea: number | null | undefined;
    readonly currentRoom: string | null | undefined;
    readonly fanSpeed: string | null | undefined;
    readonly id: string;
    readonly kind: RobotVacuumKind;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly room: string | null | undefined;
    readonly status: string | null | undefined;
  } | {
    // This will never be '%other', but we need some
    // value in case none of the concrete values match.
    readonly __typename: "%other";
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
  "name": "lastSeen",
  "storageKey": null
},
v6 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "batteryPercentage",
  "storageKey": null
},
v7 = [
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
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "on",
            "storageKey": null
          },
          (v5/*:: as any*/)
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
          (v5/*:: as any*/)
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
          (v5/*:: as any*/)
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
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "time",
            "storageKey": null
          },
          (v5/*:: as any*/)
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
            "name": "batteryVoltage",
            "storageKey": null
          },
          (v6/*:: as any*/),
          (v5/*:: as any*/),
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
          (v6/*:: as any*/),
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
          (v5/*:: as any*/)
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
    "selections": (v7/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "DashboardEntitiesQuery",
    "selections": (v7/*:: as any*/)
  },
  "params": {
    "cacheID": "b466cba0c20a27e4c894d1122eb4c1aa",
    "id": null,
    "metadata": {},
    "name": "DashboardEntitiesQuery",
    "operationKind": "query",
    "text": "query DashboardEntitiesQuery {\n  entitySections {\n    category\n    title\n  }\n  entities {\n    __typename\n    ... on LightEntity {\n      category\n      id\n      name\n      room\n      capabilities\n      on\n      lastSeen\n    }\n    ... on DoorEntity {\n      category\n      id\n      name\n      room\n      open\n      lastSeen\n    }\n    ... on PresenceEntity {\n      category\n      id\n      name\n      room\n      present\n      lastSeen\n    }\n    ... on EnvironmentEntity {\n      category\n      id\n      name\n      room\n      capabilities\n      temperature\n      humidity\n      pressure\n      lux\n      uvIndex\n      time\n      lastSeen\n    }\n    ... on EinkDisplayEntity {\n      category\n      id\n      name\n      room\n      batteryVoltage\n      batteryPercentage\n      lastSeen\n      config {\n        mode\n        view\n        album\n        orientation\n        refresh\n        settle\n        sleepStart\n        sleepEnd\n      }\n    }\n    ... on RobotVacuumEntity {\n      category\n      id\n      name\n      room\n      capabilities\n      kind\n      status\n      batteryPercentage\n      currentRoom\n      fanSpeed\n      currentCleanArea\n      cleanCount\n      lastSeen\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "247ddac14ecdd8f51264fed3cc9c1e28";

export default node;
