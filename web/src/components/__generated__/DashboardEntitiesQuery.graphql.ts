/**
 * @generated SignedSource<<14583794c294cb5b35c387816d290a5c>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type Capability = "COLOUR_TEMP" | "HUMIDITY" | "LUX" | "PRESSURE" | "RGB" | "TEMPERATURE" | "UV_INDEX" | "%future added value";
export type DashboardEntitiesQuery$variables = Record<PropertyKey, never>;
export type DashboardEntitiesQuery$data = {
  readonly entities: ReadonlyArray<{
    readonly __typename: "DoorEntity";
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly open: boolean | null | undefined;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "EinkDisplayEntity";
    readonly batteryPercentage: number | null | undefined;
    readonly batteryVoltage: number | null | undefined;
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "EnvironmentEntity";
    readonly capabilities: ReadonlyArray<Capability>;
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
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly on: boolean | null | undefined;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "PresenceEntity";
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly present: boolean | null | undefined;
    readonly room: string | null | undefined;
  } | {
    // This will never be '%other', but we need some
    // value in case none of the concrete values match.
    readonly __typename: "%other";
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
  "name": "id",
  "storageKey": null
},
v1 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "room",
  "storageKey": null
},
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "capabilities",
  "storageKey": null
},
v4 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "lastSeen",
  "storageKey": null
},
v5 = [
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
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "on",
            "storageKey": null
          },
          (v4/*:: as any*/)
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
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "open",
            "storageKey": null
          },
          (v4/*:: as any*/)
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
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "present",
            "storageKey": null
          },
          (v4/*:: as any*/)
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
          (v4/*:: as any*/)
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
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "batteryVoltage",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "batteryPercentage",
            "storageKey": null
          },
          (v4/*:: as any*/)
        ],
        "type": "EinkDisplayEntity",
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
    "selections": (v5/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "DashboardEntitiesQuery",
    "selections": (v5/*:: as any*/)
  },
  "params": {
    "cacheID": "ba6d4201fed3a956ab18e299ac33e46c",
    "id": null,
    "metadata": {},
    "name": "DashboardEntitiesQuery",
    "operationKind": "query",
    "text": "query DashboardEntitiesQuery {\n  entities {\n    __typename\n    ... on LightEntity {\n      id\n      name\n      room\n      capabilities\n      on\n      lastSeen\n    }\n    ... on DoorEntity {\n      id\n      name\n      room\n      open\n      lastSeen\n    }\n    ... on PresenceEntity {\n      id\n      name\n      room\n      present\n      lastSeen\n    }\n    ... on EnvironmentEntity {\n      id\n      name\n      room\n      capabilities\n      temperature\n      humidity\n      pressure\n      lux\n      uvIndex\n      time\n      lastSeen\n    }\n    ... on EinkDisplayEntity {\n      id\n      name\n      room\n      batteryVoltage\n      batteryPercentage\n      lastSeen\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "b01430a14be3037c3063ba66e5c67a3c";

export default node;
