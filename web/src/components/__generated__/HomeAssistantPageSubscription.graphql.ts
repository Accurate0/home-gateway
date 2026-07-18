/**
 * @generated SignedSource<<465050f9e06b442675f9c37150bb2cac>>
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
    "alias": null,
    "args": [
      {
        "kind": "Literal",
        "name": "filter",
        "value": "home_assistant:*"
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
      }
    ],
    "storageKey": "events(filter:\"home_assistant:*\")"
  }
];
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "HomeAssistantPageSubscription",
    "selections": (v0/*:: as any*/),
    "type": "SubscriptionRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "HomeAssistantPageSubscription",
    "selections": (v0/*:: as any*/)
  },
  "params": {
    "cacheID": "b57afb8a3f4010d0b852bb3688c55404",
    "id": null,
    "metadata": {},
    "name": "HomeAssistantPageSubscription",
    "operationKind": "subscription",
    "text": "subscription HomeAssistantPageSubscription {\n  events(filter: \"home_assistant:*\") {\n    __typename\n    ... on HomeAssistantUpdate {\n      eventId\n      state\n      entityId\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "dba10cc1c6c1b4f0572d7b0f1c37816d";

export default node;
