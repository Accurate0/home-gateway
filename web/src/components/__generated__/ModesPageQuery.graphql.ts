/**
 * @generated SignedSource<<fc1524e8ffb1e69d0a530b2db1fd6940>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type LightTarget = "LEAVE_ALONE" | "OFF" | "ON" | "%future added value";
export type Mode = "AWAY" | "GUEST" | "HOME" | "VACATION" | "%future added value";
export type ModesPageQuery$variables = Record<PropertyKey, never>;
export type ModesPageQuery$data = {
  readonly mode: {
    readonly active: Mode;
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
        readonly currentTarget: LightTarget;
        readonly deviceId: string | null | undefined;
        readonly name: string;
      }>;
      readonly minObservations?: number;
      readonly mode: Mode;
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
      "concreteType": "VacationLight",
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
          "concreteType": "VacationAction",
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
  "type": "VacationMode",
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
        "concreteType": "ModeStatus",
        "kind": "LinkedField",
        "name": "mode",
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
    "type": "QueryRoot",
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
        "concreteType": "ModeStatus",
        "kind": "LinkedField",
        "name": "mode",
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
    "cacheID": "3a31b007c500145425a23b4107cc7d12",
    "id": null,
    "metadata": {},
    "name": "ModesPageQuery",
    "operationKind": "query",
    "text": "query ModesPageQuery {\n  mode {\n    active\n    node(mode: VACATION) {\n      __typename\n      mode\n      ... on VacationMode {\n        enabled\n        window\n        jitter\n        minObservations\n        lights {\n          address\n          deviceId\n          name\n          coverage\n          currentTarget\n          actions {\n            at\n            on\n            slot\n            onFraction\n          }\n        }\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "32fe6a0a495fb78d34ea85da4c19e373";

export default node;
