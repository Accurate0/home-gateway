/**
 * @generated SignedSource<<6c3563c06f514f0da97ecd50deb768cf>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type SolarEnergyQuery$variables = {
  energyFrom: any;
  energyTo: any;
};
export type SolarEnergyQuery$data = {
  readonly " $fragmentSpreads": FragmentRefs<"SolarEnergyTabFragment">;
};
export type SolarEnergyQuery = {
  response: SolarEnergyQuery$data;
  variables: SolarEnergyQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "energyFrom"
  },
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "energyTo"
  }
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "SolarEnergyQuery",
    "selections": [
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
    "name": "SolarEnergyQuery",
    "selections": [
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
            "args": [
              {
                "fields": [
                  {
                    "kind": "Variable",
                    "name": "from",
                    "variableName": "energyFrom"
                  },
                  {
                    "kind": "Variable",
                    "name": "to",
                    "variableName": "energyTo"
                  }
                ],
                "kind": "ObjectValue",
                "name": "input"
              }
            ],
            "concreteType": "EnergyConsumption",
            "kind": "LinkedField",
            "name": "history",
            "plural": true,
            "selections": [
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "id",
                "storageKey": null
              },
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
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "time",
                "storageKey": null
              }
            ],
            "storageKey": null
          }
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "520f9c2835b3e2a011d1d8bd59403ccf",
    "id": null,
    "metadata": {},
    "name": "SolarEnergyQuery",
    "operationKind": "query",
    "text": "query SolarEnergyQuery(\n  $energyFrom: DateTime!\n  $energyTo: DateTime!\n) {\n  ...SolarEnergyTabFragment\n}\n\nfragment SolarEnergyTabFragment on QueryRoot {\n  solar {\n    current {\n      todayProductionKwh\n      yesterdayProductionKwh\n    }\n    history {\n      at\n      uvLevel\n      wh\n      timestamp\n    }\n  }\n  energy {\n    history(input: {from: $energyFrom, to: $energyTo}) {\n      id\n      used\n      solarExported\n      time\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "cb39d32fedcc53be49c7c731911a5732";

export default node;
