/**
 * @generated SignedSource<<cde074b6b6e672f9f56af2ac8fb55be9>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type AppQuery$variables = {
  since: any;
};
export type AppQuery$data = {
  readonly " $fragmentSpreads": FragmentRefs<"OverviewTabFragment" | "SolarEnergyTabFragment">;
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
v1 = [
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
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "time",
  "storageKey": null
},
v4 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "state",
  "storageKey": null
},
v5 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v6 = [
  (v2/*: any*/),
  (v3/*: any*/),
  (v5/*: any*/),
  (v4/*: any*/)
],
v7 = [
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
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "AppQuery",
    "selections": [
      {
        "args": null,
        "kind": "FragmentSpread",
        "name": "OverviewTabFragment"
      },
      {
        "args": null,
        "kind": "FragmentSpread",
        "name": "SolarEnergyTabFragment"
      }
    ],
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Operation",
    "name": "AppQuery",
    "selections": [
      {
        "alias": null,
        "args": (v1/*: any*/),
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
              (v2/*: any*/),
              (v3/*: any*/),
              (v4/*: any*/),
              (v5/*: any*/)
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
            "selections": (v6/*: any*/),
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "concreteType": "WifiEvent",
            "kind": "LinkedField",
            "name": "wifi",
            "plural": true,
            "selections": (v6/*: any*/),
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
            "selections": (v7/*: any*/),
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "concreteType": "EnvironmentDetails",
            "kind": "LinkedField",
            "name": "laundry",
            "plural": false,
            "selections": (v7/*: any*/),
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "concreteType": "EnvironmentDetails",
            "kind": "LinkedField",
            "name": "livingRoom",
            "plural": false,
            "selections": (v7/*: any*/),
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "concreteType": "EnvironmentDetails",
            "kind": "LinkedField",
            "name": "bedroom",
            "plural": false,
            "selections": (v7/*: any*/),
            "storageKey": null
          }
        ],
        "storageKey": null
      },
      {
        "alias": null,
        "args": (v1/*: any*/),
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
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "yesterdayProductionKwh",
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
        "args": null,
        "concreteType": "EnergyObject",
        "kind": "LinkedField",
        "name": "energy",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": (v1/*: any*/),
            "concreteType": "EnergyConsumption",
            "kind": "LinkedField",
            "name": "history",
            "plural": true,
            "selections": [
              (v5/*: any*/),
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "used",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "solarExported",
                "storageKey": null
              },
              (v3/*: any*/)
            ],
            "storageKey": null
          }
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "77310b393d74701fd807c45bea660c1e",
    "id": null,
    "metadata": {},
    "name": "AppQuery",
    "operationKind": "query",
    "text": "query AppQuery(\n  $since: DateTime!\n) {\n  ...OverviewTabFragment\n  ...SolarEnergyTabFragment\n}\n\nfragment OverviewTabFragment on QueryRoot {\n  events(input: {since: $since}) {\n    doors {\n      name\n      time\n      state\n      id\n    }\n    appliances {\n      name\n      time\n      id\n      state\n    }\n    wifi {\n      name\n      time\n      id\n      state\n    }\n  }\n  environment {\n    outdoor {\n      temperature\n      humidity\n      pressure\n    }\n    laundry {\n      temperature\n      humidity\n      pressure\n    }\n    livingRoom {\n      temperature\n      humidity\n      pressure\n    }\n    bedroom {\n      temperature\n      humidity\n      pressure\n    }\n  }\n}\n\nfragment SolarEnergyTabFragment on QueryRoot {\n  solar(input: {since: $since}) {\n    current {\n      todayProductionKwh\n      yesterdayProductionKwh\n    }\n    history {\n      at\n      uvLevel\n      wh\n      timestamp\n    }\n  }\n  energy {\n    history(input: {since: $since}) {\n      id\n      used\n      solarExported\n      time\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "c90adcf3462d58d7afc918f55399d2e3";

export default node;
