/**
 * @generated SignedSource<<4e6caf895f622cadadf54cc9456a451c>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type SolarChartQuery$variables = {
  since: any;
};
export type SolarChartQuery$data = {
  readonly solar: {
    readonly history: ReadonlyArray<{
      readonly at: any;
      readonly timestamp: number;
      readonly wh: number;
    }>;
  };
};
export type SolarChartQuery = {
  response: SolarChartQuery$data;
  variables: SolarChartQuery$variables;
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
    "name": "SolarChartQuery",
    "selections": (v1/*: any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Operation",
    "name": "SolarChartQuery",
    "selections": (v1/*: any*/)
  },
  "params": {
    "cacheID": "303c43f4e4f9dab0e58897d9485bd1b9",
    "id": null,
    "metadata": {},
    "name": "SolarChartQuery",
    "operationKind": "query",
    "text": "query SolarChartQuery(\n  $since: DateTime!\n) {\n  solar(input: {since: $since}) {\n    history {\n      wh\n      at\n      timestamp\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "635ca2e5b24e7c734b771b8b733b0b91";

export default node;
