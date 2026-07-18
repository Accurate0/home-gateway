/**
 * @generated SignedSource<<3a6e6b3f1a2f408b66a2282875093411>>
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
  } | {
    readonly __typename: "EnvironmentEntity";
    readonly capabilities: ReadonlyArray<Capability>;
    readonly humidity: number | null | undefined;
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly lux: number | null | undefined;
    readonly name: string;
    readonly pressure: number | null | undefined;
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
  } | {
    readonly __typename: "PresenceEntity";
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly present: boolean | null | undefined;
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
  "name": "capabilities",
  "storageKey": null
},
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "lastSeen",
  "storageKey": null
},
v4 = [
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
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "on",
            "storageKey": null
          },
          (v3/*:: as any*/)
        ],
        "type": "LightEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "open",
            "storageKey": null
          },
          (v3/*:: as any*/)
        ],
        "type": "DoorEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "present",
            "storageKey": null
          },
          (v3/*:: as any*/)
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
          (v3/*:: as any*/)
        ],
        "type": "EnvironmentEntity",
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
    "selections": (v4/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "DashboardEntitiesQuery",
    "selections": (v4/*:: as any*/)
  },
  "params": {
    "cacheID": "a9435eefd58cd754fdad2d11dc3d60fe",
    "id": null,
    "metadata": {},
    "name": "DashboardEntitiesQuery",
    "operationKind": "query",
    "text": "query DashboardEntitiesQuery {\n  entities {\n    __typename\n    ... on LightEntity {\n      id\n      name\n      capabilities\n      on\n      lastSeen\n    }\n    ... on DoorEntity {\n      id\n      name\n      open\n      lastSeen\n    }\n    ... on PresenceEntity {\n      id\n      name\n      present\n      lastSeen\n    }\n    ... on EnvironmentEntity {\n      id\n      name\n      capabilities\n      temperature\n      humidity\n      pressure\n      lux\n      uvIndex\n      time\n      lastSeen\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "59add7be2f62a214a2646c1a6ad8e833";

export default node;
