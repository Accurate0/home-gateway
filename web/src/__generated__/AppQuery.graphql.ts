/**
 * @generated SignedSource<<d33f77d126b189c86e29fd067c1d4a78>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type ApplianceStateType = "OFF" | "ON" | "%future added value";
export type DoorState = "CLOSED" | "OPEN" | "%future added value";
export type UnifiState = "CONNECTED" | "DISCONNECTED" | "%future added value";
export type AppQuery$variables = {
  since: any;
};
export type AppQuery$data = {
  readonly environment: {
    readonly bedroom: {
      readonly humidity: number;
      readonly pressure: number;
      readonly temperature: number;
    };
    readonly laundry: {
      readonly humidity: number;
      readonly pressure: number;
      readonly temperature: number;
    };
    readonly livingRoom: {
      readonly humidity: number;
      readonly pressure: number;
      readonly temperature: number;
    };
    readonly outdoor: {
      readonly humidity: number;
      readonly pressure: number;
      readonly temperature: number;
    };
  };
  readonly events: {
    readonly appliances: ReadonlyArray<{
      readonly id: any;
      readonly name: string;
      readonly state: ApplianceStateType;
      readonly time: any;
    }>;
    readonly doors: ReadonlyArray<{
      readonly id: any;
      readonly name: string;
      readonly state: DoorState;
      readonly time: any;
    }>;
    readonly wifi: ReadonlyArray<{
      readonly id: any;
      readonly name: string;
      readonly state: UnifiState;
      readonly time: any;
    }>;
  };
  readonly solar: {
    readonly current: {
      readonly todayProductionKwh: number;
    };
    readonly history: ReadonlyArray<{
      readonly at: any;
      readonly timestamp: number;
      readonly uvLevel: number | null | undefined;
      readonly wh: number;
    }>;
  };
};
export type AppQuery = {
  response: AppQuery$data;
  variables: AppQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "since"
  }
],
v1 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "time",
  "storageKey": null
},
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "state",
  "storageKey": null
},
v4 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v5 = [
  (v1/*: any*/),
  (v2/*: any*/),
  (v4/*: any*/),
  (v3/*: any*/)
],
v6 = [
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "temperature",
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "humidity",
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "pressure",
    "storageKey": null
  }
],
v7 = [
  {
    "alias": null,
    "args": null,
    "concreteType": "SolarObject",
    "kind": "LinkedField",
    "name": "solar",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "SolarCurrentResponse",
        "kind": "LinkedField",
        "name": "current",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "todayProductionKwh",
            "storageKey": null
          }
        ],
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "GenerationHistory",
        "kind": "LinkedField",
        "name": "history",
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
            "name": "uvLevel",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "wh",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "timestamp",
            "storageKey": null
          }
        ],
        "storageKey": null
      }
    ],
    "storageKey": null
  },
  {
    "alias": null,
    "args": [
      {
        "fields": [
          {
            "kind": "Variable",
            "name": "since",
            "variableName": "since"
          }
        ],
        "kind": "ObjectValue",
        "name": "input"
      }
    ],
    "concreteType": "EventsObject",
    "kind": "LinkedField",
    "name": "events",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "DoorEvent",
        "kind": "LinkedField",
        "name": "doors",
        "plural": true,
        "selections": [
          (v1/*: any*/),
          (v2/*: any*/),
          (v3/*: any*/),
          (v4/*: any*/)
        ],
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "ApplianceEvent",
        "kind": "LinkedField",
        "name": "appliances",
        "plural": true,
        "selections": (v5/*: any*/),
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "WifiEvent",
        "kind": "LinkedField",
        "name": "wifi",
        "plural": true,
        "selections": (v5/*: any*/),
        "storageKey": null
      }
    ],
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "concreteType": "EnvironmentObject",
    "kind": "LinkedField",
    "name": "environment",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "EnvironmentDetails",
        "kind": "LinkedField",
        "name": "outdoor",
        "plural": false,
        "selections": (v6/*: any*/),
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "EnvironmentDetails",
        "kind": "LinkedField",
        "name": "laundry",
        "plural": false,
        "selections": (v6/*: any*/),
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "EnvironmentDetails",
        "kind": "LinkedField",
        "name": "livingRoom",
        "plural": false,
        "selections": (v6/*: any*/),
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "EnvironmentDetails",
        "kind": "LinkedField",
        "name": "bedroom",
        "plural": false,
        "selections": (v6/*: any*/),
        "storageKey": null
      }
    ],
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "AppQuery",
    "selections": (v7/*: any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Operation",
    "name": "AppQuery",
    "selections": (v7/*: any*/)
  },
  "params": {
    "cacheID": "21ce2320b62e438db8ea20d6f3a59b69",
    "id": null,
    "metadata": {},
    "name": "AppQuery",
    "operationKind": "query",
    "text": "query AppQuery(\n  $since: DateTime!\n) {\n  solar {\n    current {\n      todayProductionKwh\n    }\n    history {\n      at\n      uvLevel\n      wh\n      timestamp\n    }\n  }\n  events(input: {since: $since}) {\n    doors {\n      name\n      time\n      state\n      id\n    }\n    appliances {\n      name\n      time\n      id\n      state\n    }\n    wifi {\n      name\n      time\n      id\n      state\n    }\n  }\n  environment {\n    outdoor {\n      temperature\n      humidity\n      pressure\n    }\n    laundry {\n      temperature\n      humidity\n      pressure\n    }\n    livingRoom {\n      temperature\n      humidity\n      pressure\n    }\n    bedroom {\n      temperature\n      humidity\n      pressure\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "4a5f1d3f2052ea06e6351224eb0a6c9e";

export default node;
