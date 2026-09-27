/**
 * @generated SignedSource<<596141acb3dca4dfc6d2d0bbd9c7ad6e>>
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
    "cacheID": "822ce5c881f3464658db5e210c75bbb8",
    "id": null,
    "metadata": {},
    "name": "DashboardSetColourMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetColourMutation(\n  $id: IdOrAlias!\n  $hex: String!\n) {\n  light(id: $id) {\n    setColour(input: {hex: $hex})\n  }\n}\n"
  }
};
})();

(node as any).hash = "d1462043134ce783301108bc6f3f828d";

export default node;
