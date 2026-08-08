/**
 * @generated SignedSource<<70b1ddae247269ff3bdd8e0d4b9f2a50>>
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
    readonly __typename: "MediaPlayerUpdate";
    readonly appName: string | null | undefined;
    readonly artworkUrl: string | null | undefined;
    readonly durationSeconds: number | null | undefined;
    readonly episode: number | null | undefined;
    readonly id: string;
    readonly mediaSeriesTitle: string | null | undefined;
    readonly mediaTitle: string | null | undefined;
    readonly muted: boolean | null | undefined;
    readonly name: string;
    readonly positionSeconds: number | null | undefined;
    readonly room: string | null | undefined;
    readonly season: number | null | undefined;
    readonly source: string | null | undefined;
    readonly state: string;
    readonly volumeLevel: number | null | undefined;
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
},
v8 = {
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    (v3/*:: as any*/),
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "room",
      "storageKey": null
    },
    {
      "alias": "state",
      "args": null,
      "kind": "ScalarField",
      "name": "entityState",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "appName",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "source",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "mediaTitle",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "mediaSeriesTitle",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "season",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "episode",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "positionSeconds",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "durationSeconds",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "volumeLevel",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "muted",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "artworkUrl",
      "storageKey": null
    }
  ],
  "type": "MediaPlayerUpdate",
  "abstractKey": null
},
v9 = [
  (v2/*:: as any*/)
];
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
          (v7/*:: as any*/),
          (v8/*:: as any*/)
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
          (v8/*:: as any*/),
          {
            "kind": "InlineFragment",
            "selections": (v9/*:: as any*/),
            "type": "DeviceBatteryUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v9/*:: as any*/),
            "type": "HomeAssistantUpdate",
            "abstractKey": null
          }
        ],
        "storageKey": "events(filter:\"*\")"
      }
    ]
  },
  "params": {
    "cacheID": "f6db8d639be6810619ab366dbcf64f56",
    "id": null,
    "metadata": {},
    "name": "DashboardEventsSubscription",
    "operationKind": "subscription",
    "text": "subscription DashboardEventsSubscription {\n  events(filter: \"*\") {\n    __typename\n    ... on LightUpdate {\n      id\n      name\n      on\n    }\n    ... on DoorUpdate {\n      id\n      name\n      open\n    }\n    ... on PresenceUpdate {\n      id\n      name\n      present\n    }\n    ... on EnvironmentUpdate {\n      id\n      name\n      readings {\n        metric\n        value\n      }\n    }\n    ... on MediaPlayerUpdate {\n      id\n      name\n      room\n      state: entityState\n      appName\n      source\n      mediaTitle\n      mediaSeriesTitle\n      season\n      episode\n      positionSeconds\n      durationSeconds\n      volumeLevel\n      muted\n      artworkUrl\n    }\n    ... on DeviceBatteryUpdate {\n      id\n    }\n    ... on HomeAssistantUpdate {\n      id\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "7c486af8dd8c639e824611297fe5f59e";

export default node;
