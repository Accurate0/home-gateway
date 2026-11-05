/**
 * @generated SignedSource<<26b44f60ec5d816dde00c46597296d16>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type PlantsPageQuery$variables = {
  since: any;
};
export type PlantsPageQuery$data = {
  readonly homeEntities: ReadonlyArray<{
    readonly __typename: "HomePlantEntity";
    readonly history: ReadonlyArray<{
      readonly soilMoisture: number;
      readonly time: any;
    }>;
    readonly id: string;
    readonly name: string;
    readonly room: string | null | undefined;
    readonly soilMoisture: number | null | undefined;
    readonly time: any | null | undefined;
  } | {
    // This will never be '%other', but we need some
    // value in case none of the concrete values match.
    readonly __typename: "%other";
  }>;
};
export type PlantsPageQuery = {
  response: PlantsPageQuery$data;
  variables: PlantsPageQuery$variables;
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
  "name": "soilMoisture",
  "storageKey": null
},
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "time",
  "storageKey": null
},
v3 = [
  {
    "alias": null,
    "args": null,
    "concreteType": null,
    "kind": "LinkedField",
    "name": "homeEntities",
    "plural": true,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "__typename",
        "storageKey": null
      },
      {
        "kind": "InlineFragment",
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
            "name": "name",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "room",
            "storageKey": null
          },
          (v1/*:: as any*/),
          (v2/*:: as any*/),
          {
            "alias": null,
            "args": [
              {
                "kind": "Variable",
                "name": "since",
                "variableName": "since"
              }
            ],
            "concreteType": "HomePlantPoint",
            "kind": "LinkedField",
            "name": "history",
            "plural": true,
            "selections": [
              (v2/*:: as any*/),
              (v1/*:: as any*/)
            ],
            "storageKey": null
          }
        ],
        "type": "HomePlantEntity",
        "abstractKey": null
      }
    ],
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "PlantsPageQuery",
    "selections": (v3/*:: as any*/),
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "PlantsPageQuery",
    "selections": (v3/*:: as any*/)
  },
  "params": {
    "cacheID": "be0bb0d8eb09a91409894c5d6841c42b",
    "id": null,
    "metadata": {},
    "name": "PlantsPageQuery",
    "operationKind": "query",
    "text": "query PlantsPageQuery(\n  $since: DateTime!\n) {\n  homeEntities {\n    __typename\n    ... on HomePlantEntity {\n      id\n      name\n      room\n      soilMoisture\n      time\n      history(since: $since) {\n        time\n        soilMoisture\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "bd64efc306a9934436c3a3791b97b6cb";

export default node;
