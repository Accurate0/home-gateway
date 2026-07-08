/**
 * @generated SignedSource<<8ae31adc900ff123f4871776eb60107b>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AppSetColourMutation$variables = {
  hex: string;
  id: string;
};
export type AppSetColourMutation$data = {
  readonly light: {
    readonly setColour: boolean;
  };
};
export type AppSetColourMutation = {
  response: AppSetColourMutation$data;
  variables: AppSetColourMutation$variables;
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
    "name": "AppSetColourMutation",
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
    "name": "AppSetColourMutation",
    "selections": (v2/*:: as any*/)
  },
  "params": {
    "cacheID": "40d79934511951cf51671899c041b170",
    "id": null,
    "metadata": {},
    "name": "AppSetColourMutation",
    "operationKind": "mutation",
    "text": "mutation AppSetColourMutation(\n  $id: String!\n  $hex: String!\n) {\n  light(id: $id) {\n    setColour(input: {hex: $hex})\n  }\n}\n"
  }
};
})();

(node as any).hash = "e8daaec91da83f7e3d0908f55784a89f";

export default node;
