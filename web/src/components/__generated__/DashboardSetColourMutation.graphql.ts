/**
 * @generated SignedSource<<de7ff9bb3b016832f6ccc855c3601086>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardSetColourMutation$variables = {
  hex: string;
  id: string;
};
export type DashboardSetColourMutation$data = {
  readonly light: {
    readonly setColour: boolean;
  };
};
export type DashboardSetColourMutation = {
  response: DashboardSetColourMutation$data;
  variables: DashboardSetColourMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "hex"
},
v1 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "id"
},
v2 = [
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
                "name": "hex",
                "variableName": "hex"
              }
            ],
            "kind": "ObjectValue",
            "name": "input"
          }
        ],
        "kind": "ScalarField",
        "name": "setColour",
        "storageKey": null
      }
    ],
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": [
      (v0/*:: as any*/),
      (v1/*:: as any*/)
    ],
    "kind": "Fragment",
    "metadata": null,
    "name": "DashboardSetColourMutation",
    "selections": (v2/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [
      (v1/*:: as any*/),
      (v0/*:: as any*/)
    ],
    "kind": "Operation",
    "name": "DashboardSetColourMutation",
    "selections": (v2/*:: as any*/)
  },
  "params": {
    "cacheID": "edf24a80abb277df260a808c469b8e4f",
    "id": null,
    "metadata": {},
    "name": "DashboardSetColourMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetColourMutation(\n  $id: String!\n  $hex: String!\n) {\n  light(id: $id) {\n    setColour(input: {hex: $hex})\n  }\n}\n"
  }
};
})();

(node as any).hash = "b1e66321850661f35ea7feeccae90ddc";

export default node;
