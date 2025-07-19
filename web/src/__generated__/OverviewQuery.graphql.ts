/**
 * @generated SignedSource<<fd39d792501f81ec33a10a1ea88cd35b>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type OverviewQuery$variables = {
  since: any;
};
export type OverviewQuery$data = {
  readonly " $fragmentSpreads": FragmentRefs<"OverviewTabFragment">;
};
export type OverviewQuery = {
  response: OverviewQuery$data;
  variables: OverviewQuery$variables;
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
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "OverviewQuery",
    "selections": [
      {
        "args": null,
        "kind": "FragmentSpread",
        "name": "OverviewTabFragment"
      }
    ],
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Operation",
    "name": "OverviewQuery",
    "selections": [
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
    ]
  },
  "params": {
    "cacheID": "dc8c3627e425542c1c80e20029d8678d",
    "id": null,
    "metadata": {},
    "name": "OverviewQuery",
    "operationKind": "query",
    "text": "query OverviewQuery(\n  $since: DateTime!\n) {\n  ...OverviewTabFragment\n}\n\nfragment OverviewTabFragment on QueryRoot {\n  events(input: {since: $since}) {\n    doors {\n      name\n      time\n      state\n      id\n    }\n    appliances {\n      name\n      time\n      id\n      state\n    }\n    wifi {\n      name\n      time\n      id\n      state\n    }\n  }\n  environment {\n    outdoor {\n      temperature\n      humidity\n      pressure\n    }\n    laundry {\n      temperature\n      humidity\n      pressure\n    }\n    livingRoom {\n      temperature\n      humidity\n      pressure\n    }\n    bedroom {\n      temperature\n      humidity\n      pressure\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "de344060f302228125040acdd742c645";

export default node;
