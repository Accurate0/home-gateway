/**
 * @generated SignedSource<<59e92d03a55a1ec7d5489cb5faff63fc>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type Mode = "AWAY" | "GUEST" | "HOME" | "VACATION" | "%future added value";
export type ModesPageSetModeMutation$variables = {
  mode: Mode;
};
export type ModesPageSetModeMutation$data = {
  readonly setMode: Mode;
};
export type ModesPageSetModeMutation = {
  response: ModesPageSetModeMutation$data;
  variables: ModesPageSetModeMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "mode"
  }
],
v1 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Variable",
        "name": "mode",
        "variableName": "mode"
      }
    ],
    "kind": "ScalarField",
    "name": "setMode",
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "ModesPageSetModeMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "ModesPageSetModeMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "181b036b7842c29344203a22bc9b8646",
    "id": null,
    "metadata": {},
    "name": "ModesPageSetModeMutation",
    "operationKind": "mutation",
    "text": "mutation ModesPageSetModeMutation(\n  $mode: Mode!\n) {\n  setMode(mode: $mode)\n}\n"
  }
};
})();

(node as any).hash = "3dff2f4b11dd50d42fb6f082a075a2d2";

export default node;
