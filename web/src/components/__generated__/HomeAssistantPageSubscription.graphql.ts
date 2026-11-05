/**
 * @generated SignedSource<<fb3dbe0307b929513979637825dffccd>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeAssistantPageSubscription$variables = Record<PropertyKey, never>;
export type HomeAssistantPageSubscription$data = {
  readonly homeEvents: {
    readonly __typename: "HomeHomeAssistantUpdate";
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
  "type": "HomeHomeAssistantUpdate",
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
        "name": "homeEvents",
        "plural": false,
        "selections": [
          (v1/*:: as any*/),
          (v3/*:: as any*/)
        ],
        "storageKey": "homeEvents(filter:\"home_assistant:*\")"
      }
    ],
    "type": "Subscription",
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
        "name": "homeEvents",
        "plural": false,
        "selections": [
          (v1/*:: as any*/),
          (v3/*:: as any*/),
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeAirPurifierUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeCommandFailedUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeDeviceBatteryUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeDeviceConnectionUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeDeviceLockUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeDoorUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeEnvironmentUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeGarageDoorUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeJellyfinUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeLightUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomeMediaPlayerUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomePlantUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v4/*:: as any*/),
            "type": "HomePresenceUpdate",
            "abstractKey": null
          }
        ],
        "storageKey": "homeEvents(filter:\"home_assistant:*\")"
      }
    ]
  },
  "params": {
    "cacheID": "9247c7a6b470bc7e277bf2ae1c15b900",
    "id": null,
    "metadata": {},
    "name": "HomeAssistantPageSubscription",
    "operationKind": "subscription",
    "text": "subscription HomeAssistantPageSubscription {\n  homeEvents(filter: \"home_assistant:*\") {\n    __typename\n    ... on HomeHomeAssistantUpdate {\n      id\n      eventId\n      state\n      entityId\n    }\n    ... on HomeAirPurifierUpdate {\n      id\n    }\n    ... on HomeCommandFailedUpdate {\n      id\n    }\n    ... on HomeDeviceBatteryUpdate {\n      id\n    }\n    ... on HomeDeviceConnectionUpdate {\n      id\n    }\n    ... on HomeDeviceLockUpdate {\n      id\n    }\n    ... on HomeDoorUpdate {\n      id\n    }\n    ... on HomeEnvironmentUpdate {\n      id\n    }\n    ... on HomeGarageDoorUpdate {\n      id\n    }\n    ... on HomeJellyfinUpdate {\n      id\n    }\n    ... on HomeLightUpdate {\n      id\n    }\n    ... on HomeMediaPlayerUpdate {\n      id\n    }\n    ... on HomePlantUpdate {\n      id\n    }\n    ... on HomePresenceUpdate {\n      id\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "047bf4fe6d191d9ae39ea9bdf4609e49";

export default node;
