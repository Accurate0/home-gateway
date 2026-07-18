/**
 * @generated SignedSource<<c3a76af856c380f4d85521ffe19892bf>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardEventsSubscription$variables = Record<PropertyKey, never>;
export type DashboardEventsSubscription$data = {
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
export type DashboardEventsSubscription = {
  response: DashboardEventsSubscription$data;
  variables: DashboardEventsSubscription$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "kind": "Literal",
    "name": "filter",
    "value": "*"
  }
],
v1 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "__typename",
  "storageKey": null
},
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v4 = {
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    (v3/*:: as any*/),
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
v5 = {
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    (v3/*:: as any*/),
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
v6 = {
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    (v3/*:: as any*/),
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
v7 = {
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    (v3/*:: as any*/),
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
};
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "DashboardEventsSubscription",
    "selections": [
      {
        "alias": null,
        "args": (v0/*:: as any*/),
        "concreteType": null,
        "kind": "LinkedField",
        "name": "events",
        "plural": false,
        "selections": [
          (v1/*:: as any*/),
          (v4/*:: as any*/),
          (v5/*:: as any*/),
          (v6/*:: as any*/),
          (v7/*:: as any*/)
        ],
        "storageKey": "events(filter:\"*\")"
      }
    ],
    "type": "SubscriptionRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "DashboardEventsSubscription",
    "selections": [
      {
        "alias": null,
        "args": (v0/*:: as any*/),
        "concreteType": null,
        "kind": "LinkedField",
        "name": "events",
        "plural": false,
        "selections": [
          (v1/*:: as any*/),
          (v4/*:: as any*/),
          (v5/*:: as any*/),
          (v6/*:: as any*/),
          (v7/*:: as any*/),
          {
            "kind": "InlineFragment",
            "selections": [
              (v2/*:: as any*/)
            ],
            "type": "HomeAssistantUpdate",
            "abstractKey": null
          }
        ],
        "storageKey": "events(filter:\"*\")"
      }
    ]
  },
  "params": {
    "cacheID": "04e31d8b37c913b7efe1a4deeea304b6",
    "id": null,
    "metadata": {},
    "name": "DashboardEventsSubscription",
    "operationKind": "subscription",
    "text": "subscription DashboardEventsSubscription {\n  events(filter: \"*\") {\n    __typename\n    ... on LightUpdate {\n      id\n      name\n      on\n    }\n    ... on DoorUpdate {\n      id\n      name\n      open\n    }\n    ... on PresenceUpdate {\n      id\n      name\n      present\n    }\n    ... on EnvironmentUpdate {\n      id\n      name\n      readings {\n        metric\n        value\n      }\n    }\n    ... on HomeAssistantUpdate {\n      id\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "d7b58500f402eaa2d07f5b7851b4542c";

export default node;
