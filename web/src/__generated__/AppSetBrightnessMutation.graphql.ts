/**
 * @generated SignedSource<<de00c007c3adb8a81b6d621afc5e5828>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AppSetBrightnessMutation$variables = {
  id: string;
  value: number;
};
export type AppSetBrightnessMutation$data = {
  readonly light: {
    readonly setBrightness: boolean;
  };
};
export type AppSetBrightnessMutation = {
  response: AppSetBrightnessMutation$data;
  variables: AppSetBrightnessMutation$variables;
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
        "name": "setBrightness",
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
    "name": "AppSetBrightnessMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "AppSetBrightnessMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "58645ef8ec11b71a7b549736dc650398",
    "id": null,
    "metadata": {},
    "name": "AppSetBrightnessMutation",
    "operationKind": "mutation",
    "text": "mutation AppSetBrightnessMutation(\n  $id: String!\n  $value: Int!\n) {\n  light(id: $id) {\n    setBrightness(input: {value: $value})\n  }\n}\n"
  }
};
})();

(node as any).hash = "300efd91e061323a01e7a9f21881ef16";

export default node;
