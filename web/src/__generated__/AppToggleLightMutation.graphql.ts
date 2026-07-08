/**
 * @generated SignedSource<<9bf2860f25e01717e0b7bde62e4cc7da>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AppToggleLightMutation$variables = {
  id: string;
};
export type AppToggleLightMutation$data = {
  readonly light: {
    readonly toggle: boolean;
  };
};
export type AppToggleLightMutation = {
  response: AppToggleLightMutation$data;
  variables: AppToggleLightMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "id"
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
        "args": null,
        "kind": "ScalarField",
        "name": "toggle",
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
    "name": "AppToggleLightMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "AppToggleLightMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "a7d119cce78f0c50c0ebf5d2111bb5fc",
    "id": null,
    "metadata": {},
    "name": "AppToggleLightMutation",
    "operationKind": "mutation",
    "text": "mutation AppToggleLightMutation(\n  $id: String!\n) {\n  light(id: $id) {\n    toggle\n  }\n}\n"
  }
};
})();

(node as any).hash = "1f235a79a14b508717fe2c0933086019";

export default node;
