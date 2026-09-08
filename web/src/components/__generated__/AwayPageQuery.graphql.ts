/**
 * @generated SignedSource<<ee4d3dd4b9980e8615c941f8907e3764>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type LightTarget = "LEAVE_ALONE" | "OFF" | "ON" | "%future added value";
export type Mode = "AWAY" | "GUEST" | "HOME" | "NIGHT" | "PARTY" | "VACATION" | "%future added value";
export type AwayPageQuery$variables = Record<PropertyKey, never>;
export type AwayPageQuery$data = {
  readonly modes: ReadonlyArray<{
    readonly active: boolean;
    readonly enabled?: boolean;
    readonly jitter?: string;
    readonly lights?: ReadonlyArray<{
      readonly actions: ReadonlyArray<{
        readonly at: any;
        readonly on: boolean;
        readonly onFraction: number;
        readonly slot: number;
      }>;
      readonly address: string;
      readonly coverage: number;
      readonly currentTarget: LightTarget;
      readonly deviceId: string | null | undefined;
      readonly name: string;
    }>;
    readonly minObservations?: number;
    readonly mode: Mode;
    readonly window?: string;
  }>;
};
export type AwayPageQuery = {
  response: AwayPageQuery$data;
  variables: AwayPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "mode",
  "storageKey": null
},
v1 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "active",
  "storageKey": null
},
v2 = {
  "kind": "InlineFragment",
  "selections": [
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "enabled",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "window",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "jitter",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "minObservations",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "concreteType": "AwayLight",
      "kind": "LinkedField",
      "name": "lights",
      "plural": true,
      "selections": [
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "address",
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "deviceId",
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "name",
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "coverage",
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "currentTarget",
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "concreteType": "AwayAction",
          "kind": "LinkedField",
          "name": "actions",
          "plural": true,
          "selections": [
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "at",
              "storageKey": null
            },
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "on",
              "storageKey": null
            },
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "slot",
              "storageKey": null
            },
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "onFraction",
              "storageKey": null
            }
          ],
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "type": "AwayMode",
  "abstractKey": null
};
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "AwayPageQuery",
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": null,
        "kind": "LinkedField",
        "name": "modes",
        "plural": true,
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/)
        ],
        "storageKey": null
      }
    ],
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "AwayPageQuery",
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": null,
        "kind": "LinkedField",
        "name": "modes",
        "plural": true,
        "selections": [
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "__typename",
            "storageKey": null
          },
          (v0/*:: as any*/),
          (v1/*:: as any*/),
          (v2/*:: as any*/)
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "12aa50582fc13d1c6edc16714e6699bc",
    "id": null,
    "metadata": {},
    "name": "AwayPageQuery",
    "operationKind": "query",
    "text": "query AwayPageQuery {\n  modes {\n    __typename\n    mode\n    active\n    ... on AwayMode {\n      enabled\n      window\n      jitter\n      minObservations\n      lights {\n        address\n        deviceId\n        name\n        coverage\n        currentTarget\n        actions {\n          at\n          on\n          slot\n          onFraction\n        }\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "fe51d84c79f42cce670daba2ce30db68";

export default node;
