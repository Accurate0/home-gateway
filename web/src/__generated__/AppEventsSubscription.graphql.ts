/**
 * @generated SignedSource<<665150d7691679ff1d1610bf81932489>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AppEventsSubscription$variables = Record<PropertyKey, never>;
export type AppEventsSubscription$data = {
  readonly events: {
    readonly __typename: "DoorUpdate";
    readonly id: string;
    readonly name: string;
    readonly open: boolean;
  } | {
    readonly __typename: "EnvironmentUpdate";
    readonly id: string;
    readonly name: string;
    readonly readings: ReadonlyArray<{
      readonly metric: string;
      readonly value: number;
    }>;
  } | {
    readonly __typename: "LightUpdate";
    readonly id: string;
    readonly name: string;
    readonly on: boolean;
  } | {
    readonly __typename: "PresenceUpdate";
    readonly id: string;
    readonly name: string;
    readonly present: boolean;
  } | {
    // This will never be '%other', but we need some
    // value in case none of the concrete values match.
    readonly __typename: "%other";
  };
};
export type AppEventsSubscription = {
  response: AppEventsSubscription$data;
  variables: AppEventsSubscription$variables;
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
v2 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Literal",
        "name": "filter",
        "value": "*"
      }
    ],
    "concreteType": null,
    "kind": "LinkedField",
    "name": "events",
    "plural": false,
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
            "name": "on",
            "storageKey": null
          }
        ],
        "type": "LightUpdate",
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
          }
        ],
        "type": "DoorUpdate",
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
          }
        ],
        "type": "PresenceUpdate",
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
            "concreteType": "MetricReading",
            "kind": "LinkedField",
            "name": "readings",
            "plural": true,
            "selections": [
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "metric",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "value",
                "storageKey": null
              }
            ],
            "storageKey": null
          }
        ],
        "type": "EnvironmentUpdate",
        "abstractKey": null
      }
    ],
    "storageKey": "events(filter:\"*\")"
  }
];
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "AppEventsSubscription",
    "selections": (v2/*:: as any*/),
    "type": "SubscriptionRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "AppEventsSubscription",
    "selections": (v2/*:: as any*/)
  },
  "params": {
    "cacheID": "347661cb2b9651611f91468be963e9db",
    "id": null,
    "metadata": {},
    "name": "AppEventsSubscription",
    "operationKind": "subscription",
    "text": "subscription AppEventsSubscription {\n  events(filter: \"*\") {\n    __typename\n    ... on LightUpdate {\n      id\n      name\n      on\n    }\n    ... on DoorUpdate {\n      id\n      name\n      open\n    }\n    ... on PresenceUpdate {\n      id\n      name\n      present\n    }\n    ... on EnvironmentUpdate {\n      id\n      name\n      readings {\n        metric\n        value\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "2fc029764c02b1547819b5741abf1071";

export default node;
