/**
 * @generated SignedSource<<7d5b80d5e7675adfa6ddf5c77a0bb6ee>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type SolarRelayChartQuery$variables = {
  since: any;
};
export type SolarRelayChartQuery$data = {
  readonly solar: {
    readonly history: ReadonlyArray<{
      readonly at: any;
      readonly timestamp: number;
      readonly wh: number;
    }>;
  };
};
export type SolarRelayChartQuery = {
  response: SolarRelayChartQuery$data;
  variables: SolarRelayChartQuery$variables;
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
    "concreteType": "SolarObject",
    "kind": "LinkedField",
    "name": "solar",
    "plural": false,
    "selections": [
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
            "name": "wh",
            "storageKey": null
          },
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
            "name": "timestamp",
            "storageKey": null
          }
        ],
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
    "name": "SolarRelayChartQuery",
    "selections": (v1/*: any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Operation",
    "name": "SolarRelayChartQuery",
    "selections": (v1/*: any*/)
  },
  "params": {
    "cacheID": "8f7eb47b43a88bbc70862805e5277ab7",
    "id": null,
    "metadata": {},
    "name": "SolarRelayChartQuery",
    "operationKind": "query",
    "text": "query SolarRelayChartQuery(\n  $since: DateTime!\n) {\n  solar(input: {since: $since}) {\n    history {\n      wh\n      at\n      timestamp\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "5c5dcf325646379c2cf25fd85578e5e7";

export default node;
