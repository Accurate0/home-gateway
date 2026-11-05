/**
 * @generated SignedSource<<ba97659dd9550b2ff75134514debb587>>
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
  readonly homeLight: {
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
    "type": "Mutation",
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
    "cacheID": "242c4889c6440bd33714d4be8540f605",
    "id": null,
    "metadata": {},
    "name": "DashboardSetColourMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetColourMutation(\n  $id: IdOrAlias!\n  $hex: String!\n) {\n  homeLight(id: $id) {\n    setColour(input: {hex: $hex})\n  }\n}\n"
  }
};
})();

(node as any).hash = "39febe866cf0e51c7e0dcabea23d2958";

export default node;
