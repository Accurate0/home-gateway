/**
 * @generated SignedSource<<734b2730265530a211be3e0c5b6a8fed>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeAirPurifierMode = "AUTO" | "MANUAL" | "SLEEP" | "%future added value";
export type HomeDeviceLockSource = "MANUAL" | "PHYSICAL" | "%future added value";
export type HomeGarageDoorState = "CLOSED" | "CLOSING" | "OPEN" | "OPENING" | "%future added value";
export type DashboardEventsSubscription$variables = Record<PropertyKey, never>;
export type DashboardEventsSubscription$data = {
  readonly homeEvents: {
    readonly __typename: "HomeAirPurifierUpdate";
    readonly displayOn: boolean | null | undefined;
    readonly id: string;
    readonly name: string;
    readonly on: boolean;
    readonly purifierMode: HomeAirPurifierMode | null | undefined;
    readonly purifierSpeed: number | null | undefined;
  } | {
    readonly __typename: "HomeDeviceLockUpdate";
    readonly expiresAt: any | null | undefined;
    readonly id: string;
    readonly lockSource: HomeDeviceLockSource | null | undefined;
    readonly locked: boolean;
  } | {
    readonly __typename: "HomeDoorUpdate";
    readonly id: string;
    readonly name: string;
    readonly open: boolean;
  } | {
    readonly __typename: "HomeEnvironmentUpdate";
    readonly id: string;
    readonly name: string;
    readonly readings: ReadonlyArray<{
      readonly metric: string;
      readonly value: number;
    }>;
  } | {
    readonly __typename: "HomeGarageDoorUpdate";
    readonly garageState: HomeGarageDoorState;
    readonly id: string;
    readonly name: string;
  } | {
    readonly __typename: "HomeLightUpdate";
    readonly id: string;
    readonly name: string;
    readonly on: boolean;
  } | {
    readonly __typename: "HomeMediaPlayerUpdate";
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
    readonly __typename: "HomePlantUpdate";
    readonly id: string;
    readonly name: string;
    readonly soilMoisture: number;
  } | {
    readonly __typename: "HomePresenceUpdate";
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
  "type": "HomeLightUpdate",
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
  "type": "HomeDoorUpdate",
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
  "type": "HomeGarageDoorUpdate",
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
  "type": "HomeAirPurifierUpdate",
  "abstractKey": null
},
v9 = {
  "kind": "InlineFragment",
  "selections": [
    (v2/*:: as any*/),
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "locked",
      "storageKey": null
    },
    {
      "alias": "lockSource",
      "args": null,
      "kind": "ScalarField",
      "name": "source",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "expiresAt",
      "storageKey": null
    }
  ],
  "type": "HomeDeviceLockUpdate",
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
      "kind": "ScalarField",
      "name": "present",
      "storageKey": null
    }
  ],
  "type": "HomePresenceUpdate",
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
      "concreteType": "HomeMetricReading",
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
  "type": "HomeEnvironmentUpdate",
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
      "name": "soilMoisture",
      "storageKey": null
    }
  ],
  "type": "HomePlantUpdate",
  "abstractKey": null
},
v13 = {
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
  "type": "HomeMediaPlayerUpdate",
  "abstractKey": null
},
v14 = [
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
        "name": "homeEvents",
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
          (v13/*:: as any*/)
        ],
        "storageKey": "homeEvents(filter:\"*\")"
      }
    ],
    "type": "Subscription",
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
        "name": "homeEvents",
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
          (v13/*:: as any*/),
          {
            "kind": "InlineFragment",
            "selections": (v14/*:: as any*/),
            "type": "HomeCommandFailedUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v14/*:: as any*/),
            "type": "HomeDeviceBatteryUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v14/*:: as any*/),
            "type": "HomeDeviceConnectionUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v14/*:: as any*/),
            "type": "HomeHomeAssistantUpdate",
            "abstractKey": null
          },
          {
            "kind": "InlineFragment",
            "selections": (v14/*:: as any*/),
            "type": "HomeJellyfinUpdate",
            "abstractKey": null
          }
        ],
        "storageKey": "homeEvents(filter:\"*\")"
      }
    ]
  },
  "params": {
    "cacheID": "245957ba0dd5a065ecdf2e860f447076",
    "id": null,
    "metadata": {},
    "name": "DashboardEventsSubscription",
    "operationKind": "subscription",
    "text": "subscription DashboardEventsSubscription {\n  homeEvents(filter: \"*\") {\n    __typename\n    ... on HomeLightUpdate {\n      id\n      name\n      on\n    }\n    ... on HomeDoorUpdate {\n      id\n      name\n      open\n    }\n    ... on HomeGarageDoorUpdate {\n      id\n      name\n      garageState: state\n    }\n    ... on HomeAirPurifierUpdate {\n      id\n      name\n      on\n      purifierMode: mode\n      purifierSpeed: speed\n      displayOn: display\n    }\n    ... on HomeDeviceLockUpdate {\n      id\n      locked\n      lockSource: source\n      expiresAt\n    }\n    ... on HomePresenceUpdate {\n      id\n      name\n      present\n    }\n    ... on HomeEnvironmentUpdate {\n      id\n      name\n      readings {\n        metric\n        value\n      }\n    }\n    ... on HomePlantUpdate {\n      id\n      name\n      soilMoisture\n    }\n    ... on HomeMediaPlayerUpdate {\n      id\n      name\n      room\n      state: entityState\n      appName\n      source\n      mediaTitle\n      mediaSeriesTitle\n      season\n      episode\n      positionSeconds\n      durationSeconds\n      volumeLevel\n      muted\n      artworkUrl\n    }\n    ... on HomeCommandFailedUpdate {\n      id\n    }\n    ... on HomeDeviceBatteryUpdate {\n      id\n    }\n    ... on HomeDeviceConnectionUpdate {\n      id\n    }\n    ... on HomeHomeAssistantUpdate {\n      id\n    }\n    ... on HomeJellyfinUpdate {\n      id\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "6e51c251999b80ad4e63b8195d2af26c";

export default node;
