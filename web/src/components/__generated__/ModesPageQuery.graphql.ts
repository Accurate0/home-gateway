/**
 * @generated SignedSource<<69d343cbf112bf3d8ae7bdd40e6d0015>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeLightTarget = "LEAVE_ALONE" | "OFF" | "ON" | "%future added value";
export type HomeMode = "AWAY" | "GUEST" | "HOME" | "VACATION" | "%future added value";
export type ModesPageQuery$variables = Record<PropertyKey, never>;
export type ModesPageQuery$data = {
  readonly homeMode: {
    readonly active: HomeMode;
    readonly node: {
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
        readonly currentTarget: HomeLightTarget;
        readonly deviceId: string | null | undefined;
        readonly name: string;
      }>;
      readonly minObservations?: number;
      readonly mode: HomeMode;
      readonly window?: string;
    };
  };
};
export type ModesPageQuery = {
  response: ModesPageQuery$data;
  variables: ModesPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "active",
  "storageKey": null
},
v1 = [
  {
    "kind": "Literal",
    "name": "mode",
    "value": "VACATION"
  }
],
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "mode",
  "storageKey": null
},
v3 = {
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
      "concreteType": "HomeVacationLight",
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
          "concreteType": "HomeVacationAction",
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
  "type": "HomeVacationMode",
  "abstractKey": null
};
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "ModesPageQuery",
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "HomeModeStatus",
        "kind": "LinkedField",
        "name": "homeMode",
        "plural": false,
        "selections": [
          (v0/*:: as any*/),
          {
            "alias": null,
            "args": (v1/*:: as any*/),
            "concreteType": null,
            "kind": "LinkedField",
            "name": "node",
            "plural": false,
            "selections": [
              (v2/*:: as any*/),
              (v3/*:: as any*/)
            ],
            "storageKey": "node(mode:\"VACATION\")"
          }
        ],
        "storageKey": null
      }
    ],
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "ModesPageQuery",
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "HomeModeStatus",
        "kind": "LinkedField",
        "name": "homeMode",
        "plural": false,
        "selections": [
          (v0/*:: as any*/),
          {
            "alias": null,
            "args": (v1/*:: as any*/),
            "concreteType": null,
            "kind": "LinkedField",
            "name": "node",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "__typename",
                "storageKey": null
              },
              (v2/*:: as any*/),
              (v3/*:: as any*/)
            ],
            "storageKey": "node(mode:\"VACATION\")"
          }
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "0292fbd25efc5d49cece26bc47b5186d",
    "id": null,
    "metadata": {},
    "name": "ModesPageQuery",
    "operationKind": "query",
    "text": "query ModesPageQuery {\n  homeMode {\n    active\n    node(mode: VACATION) {\n      __typename\n      mode\n      ... on HomeVacationMode {\n        enabled\n        window\n        jitter\n        minObservations\n        lights {\n          address\n          deviceId\n          name\n          coverage\n          currentTarget\n          actions {\n            at\n            on\n            slot\n            onFraction\n          }\n        }\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "7254b6e7fb8014a7b9204cc034a198f3";

export default node;
