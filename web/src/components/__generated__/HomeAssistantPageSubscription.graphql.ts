/**
 * @generated SignedSource<<03119049abea63300196897be833be7b>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeAssistantPageSubscription$variables = Record<PropertyKey, never>;
export type HomeAssistantPageSubscription$data = {
  readonly events: {
    readonly __typename: "HomeAssistantUpdate";
    readonly entityId: string;
    readonly eventId: any;
    readonly id: string;
    readonly state: string;
  } | {
    // This will never be '%other', but we need some
    // value in case none of the concrete values match.
    readonly __typename: "%other";
  };
};
export type HomeAssistantPageSubscription = {
  response: HomeAssistantPageSubscription$data;
  variables: HomeAssistantPageSubscription$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "kind": "Literal",
    "name": "filter",
    "value": "home_assistant:*"
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
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "eventId",
      "storageKey": null
    },
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
      "name": "entityId",
      "storageKey": null
    }
  ],
  "type": "HomeAssistantUpdate",
  "abstractKey": null
},
v4 = [
  (v2/*:: as any*/)
];
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "HomeAssistantPageSubscription",
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
          (v3/*:: as any*/)
        ],
        "storageKey": "events(filter:\"home_assistant:*\")"
      }
    ],
    "type": "SubscriptionRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "HomeAssistantPageSubscription",
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
          (v3/*:: as any*/),
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "DoorUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "EnvironmentUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "LightUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "PresenceUpdate",
            "abstractKey": null
          }
        ],
        "storageKey": "events(filter:\"home_assistant:*\")"
      }
    ]
  },
  "params": {
    "cacheID": "45e1782f925b375db44c2b4d4ea09847",
    "id": null,
    "metadata": {},
    "name": "HomeAssistantPageSubscription",
    "operationKind": "subscription",
    "text": "subscription HomeAssistantPageSubscription {\n  events(filter: \"home_assistant:*\") {\n    __typename\n    ... on HomeAssistantUpdate {\n      id\n      eventId\n      state\n      entityId\n    }\n    ... on DoorUpdate {\n      id\n    }\n    ... on EnvironmentUpdate {\n      id\n    }\n    ... on LightUpdate {\n      id\n    }\n    ... on PresenceUpdate {\n      id\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "1e2fec4706b604497b4becbe8e7fe88e";

export default node;
