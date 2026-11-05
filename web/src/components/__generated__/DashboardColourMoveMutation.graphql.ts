/**
 * @generated SignedSource<<16e47c18a2d1aaba08032de8ebf80f22>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardColourMoveMutation$variables = {
  id: string;
  value: number;
};
export type DashboardColourMoveMutation$data = {
  readonly homeLight: {
    readonly colourTemperatureMove: boolean;
  };
};
export type DashboardColourMoveMutation = {
  response: DashboardColourMoveMutation$data;
  variables: DashboardColourMoveMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "id"
  },
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "value"
  }
],
v1 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Variable",
        "name": "id",
        "variableName": "id"
      }
    ],
    "concreteType": "HomeLightMutation",
    "kind": "LinkedField",
    "name": "homeLight",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": [
          {
            "fields": [
              {
                "kind": "Variable",
                "name": "value",
                "variableName": "value"
              }
            ],
            "kind": "ObjectValue",
            "name": "input"
          }
        ],
        "kind": "ScalarField",
        "name": "colourTemperatureMove",
        "storageKey": null
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
    "name": "DashboardColourMoveMutation",
    "selections": (v1/*:: as any*/),
    "type": "Mutation",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardColourMoveMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "2eded0e6acad0afd1fa0c31218231d5c",
    "id": null,
    "metadata": {},
    "name": "DashboardColourMoveMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardColourMoveMutation(\n  $id: IdOrAlias!\n  $value: Int!\n) {\n  homeLight(id: $id) {\n    colourTemperatureMove(input: {value: $value})\n  }\n}\n"
  }
};
})();

(node as any).hash = "34cb5a366156b079fdebd6e5939e92c2";

export default node;
