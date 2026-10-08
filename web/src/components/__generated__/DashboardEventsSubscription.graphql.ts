/**
 * @generated SignedSource<<275794c5b0fead639cd950cdb7c45ca6>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AirPurifierMode = "AUTO" | "MANUAL" | "SLEEP" | "%future added value";
export type GarageDoorState = "CLOSED" | "CLOSING" | "OPEN" | "OPENING" | "%future added value";
export type DashboardEventsSubscription$variables = Record<PropertyKey, never>;
export type DashboardEventsSubscription$data = {
  readonly events: {
    readonly __typename: "AirPurifierUpdate";
    readonly displayOn: boolean | null | undefined;
    readonly id: string;
    readonly name: string;
    readonly on: boolean;
    readonly purifierMode: AirPurifierMode | null | undefined;
    readonly purifierSpeed: number | null | undefined;
  } | {
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
    readonly __typename: "GarageDoorUpdate";
    readonly garageState: GarageDoorState;
    readonly id: string;
    readonly name: string;
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
    readonly __typename: "PlantUpdate";
    readonly id: string;
    readonly name: string;
    readonly soilMoisture: number;
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
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "on",
  "storageKey": null
},
v5 = {
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    (v3/*:: as any*/),
    (v4/*:: as any*/)
  ],
  "type": "LightUpdate",
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
      "name": "open",
      "storageKey": null
    }
  ],
  "type": "DoorUpdate",
  "abstractKey": null
},
v7 = {
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    (v3/*:: as any*/),
    {
      "alias": "garageState",
      "args": null,
      "kind": "ScalarField",
      "name": "state",
      "storageKey": null
    }
  ],
  "type": "GarageDoorUpdate",
  "abstractKey": null
},
v8 = {
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    (v3/*:: as any*/),
    (v4/*:: as any*/),
    {
      "alias": "purifierMode",
      "args": null,
      "kind": "ScalarField",
      "name": "mode",
      "storageKey": null
    },
    {
      "alias": "purifierSpeed",
      "args": null,
      "kind": "ScalarField",
      "name": "speed",
      "storageKey": null
    },
    {
      "alias": "displayOn",
      "args": null,
      "kind": "ScalarField",
      "name": "display",
      "storageKey": null
    }
  ],
  "type": "AirPurifierUpdate",
  "abstractKey": null
},
v9 = {
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
v10 = {
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
v11 = {
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    (v3/*:: as any*/),
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "soilMoisture",
      "storageKey": null
    }
  ],
  "type": "PlantUpdate",
  "abstractKey": null
},
v12 = {
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
v13 = [
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
          (v5/*:: as any*/),
          (v6/*:: as any*/),
          (v7/*:: as any*/),
          (v8/*:: as any*/),
          (v9/*:: as any*/),
          (v10/*:: as any*/),
          (v11/*:: as any*/),
          (v12/*:: as any*/)
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
          (v5/*:: as any*/),
          (v6/*:: as any*/),
          (v7/*:: as any*/),
          (v8/*:: as any*/),
          (v9/*:: as any*/),
          (v10/*:: as any*/),
          (v11/*:: as any*/),
          (v12/*:: as any*/),
          {
            "kind": "InlineFragment",
            "selections": (v13/*:: as any*/),
            "type": "CommandFailedUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v13/*:: as any*/),
            "type": "DeviceBatteryUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v13/*:: as any*/),
            "type": "DeviceConnectionUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v13/*:: as any*/),
            "type": "HomeAssistantUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v13/*:: as any*/),
            "type": "JellyfinUpdate",
            "abstractKey": null
          }
        ],
        "storageKey": "events(filter:\"*\")"
      }
    ]
  },
  "params": {
    "cacheID": "b63e1e3c63e8a20b386b81548403ef1d",
    "id": null,
    "metadata": {},
    "name": "DashboardEventsSubscription",
    "operationKind": "subscription",
    "text": "subscription DashboardEventsSubscription {\n  events(filter: \"*\") {\n    __typename\n    ... on LightUpdate {\n      id\n      name\n      on\n    }\n    ... on DoorUpdate {\n      id\n      name\n      open\n    }\n    ... on GarageDoorUpdate {\n      id\n      name\n      garageState: state\n    }\n    ... on AirPurifierUpdate {\n      id\n      name\n      on\n      purifierMode: mode\n      purifierSpeed: speed\n      displayOn: display\n    }\n    ... on PresenceUpdate {\n      id\n      name\n      present\n    }\n    ... on EnvironmentUpdate {\n      id\n      name\n      readings {\n        metric\n        value\n      }\n    }\n    ... on PlantUpdate {\n      id\n      name\n      soilMoisture\n    }\n    ... on MediaPlayerUpdate {\n      id\n      name\n      room\n      state: entityState\n      appName\n      source\n      mediaTitle\n      mediaSeriesTitle\n      season\n      episode\n      positionSeconds\n      durationSeconds\n      volumeLevel\n      muted\n      artworkUrl\n    }\n    ... on CommandFailedUpdate {\n      id\n    }\n    ... on DeviceBatteryUpdate {\n      id\n    }\n    ... on DeviceConnectionUpdate {\n      id\n    }\n    ... on HomeAssistantUpdate {\n      id\n    }\n    ... on JellyfinUpdate {\n      id\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "984dd57ecf15677491fda595f5841d9a";

export default node;
