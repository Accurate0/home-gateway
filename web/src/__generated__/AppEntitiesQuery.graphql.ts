/**
 * @generated SignedSource<<a6f9075b0309f49a053234f605d6162c>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type Capability = "COLOUR_TEMP" | "RGB" | "%future added value";
export type AppEntitiesQuery$variables = Record<PropertyKey, never>;
export type AppEntitiesQuery$data = {
  readonly entities: ReadonlyArray<{
    readonly __typename: "DoorEntity";
    readonly id: string;
    readonly lastSeen: any | null | undefined;
    readonly name: string;
    readonly open: boolean | null | undefined;
  } | {
    readonly __typename: "EnvironmentEntity";
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
export type AppEntitiesQuery = {
  response: AppEntitiesQuery$data;
  variables: AppEntitiesQuery$variables;
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
  "name": "lastSeen",
  "storageKey": null
},
v3 = [
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
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "capabilities",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "on",
            "storageKey": null
          },
          (v2/*:: as any*/)
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
          (v2/*:: as any*/)
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
          (v2/*:: as any*/)
        ],
        "type": "PresenceEntity",
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
          (v2/*:: as any*/)
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
    "name": "AppEntitiesQuery",
    "selections": (v3/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "AppEntitiesQuery",
    "selections": (v3/*:: as any*/)
  },
  "params": {
    "cacheID": "89084279fe4a54d6b51ac4990204e3bb",
    "id": null,
    "metadata": {},
    "name": "AppEntitiesQuery",
    "operationKind": "query",
    "text": "query AppEntitiesQuery {\n  entities {\n    __typename\n    ... on LightEntity {\n      id\n      name\n      capabilities\n      on\n      lastSeen\n    }\n    ... on DoorEntity {\n      id\n      name\n      open\n      lastSeen\n    }\n    ... on PresenceEntity {\n      id\n      name\n      present\n      lastSeen\n    }\n    ... on EnvironmentEntity {\n      id\n      name\n      temperature\n      humidity\n      pressure\n      lux\n      uvIndex\n      time\n      lastSeen\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "7f9e34efa965c7d2b6959824683bb765";

export default node;
