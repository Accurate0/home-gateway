/**
 * @generated SignedSource<<f42a287f43b24a177c0ce0a900867172>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AppColourMoveMutation$variables = {
  id: string;
  value: number;
};
export type AppColourMoveMutation$data = {
  readonly light: {
    readonly colourTemperatureMove: boolean;
  };
};
export type AppColourMoveMutation = {
  response: AppColourMoveMutation$data;
  variables: AppColourMoveMutation$variables;
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
    "concreteType": "LightMutation",
    "kind": "LinkedField",
    "name": "light",
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
    "name": "AppColourMoveMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "AppColourMoveMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "08e0cd52f47b0b0c4111325cef920896",
    "id": null,
    "metadata": {},
    "name": "AppColourMoveMutation",
    "operationKind": "mutation",
    "text": "mutation AppColourMoveMutation(\n  $id: String!\n  $value: Int!\n) {\n  light(id: $id) {\n    colourTemperatureMove(input: {value: $value})\n  }\n}\n"
  }
};
})();

(node as any).hash = "7c175a59b15425776ffc4cd724b192da";

export default node;
