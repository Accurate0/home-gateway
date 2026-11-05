/**
 * @generated SignedSource<<f79a5ab8a44c5b565e65c8f3c664bd62>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeMode = "AWAY" | "GUEST" | "HOME" | "VACATION" | "%future added value";
export type ModesPageSetModeMutation$variables = {
  mode: HomeMode;
};
export type ModesPageSetModeMutation$data = {
  readonly homeSetMode: HomeMode;
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
    "name": "homeSetMode",
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
    "type": "Mutation",
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
    "cacheID": "9445d0f11fe9cf1ece89a6b2e07dead0",
    "id": null,
    "metadata": {},
    "name": "ModesPageSetModeMutation",
    "operationKind": "mutation",
    "text": "mutation ModesPageSetModeMutation(\n  $mode: HomeMode!\n) {\n  homeSetMode(mode: $mode)\n}\n"
  }
};
})();

(node as any).hash = "9a8ae860a2d562f14673e4315f9349ae";

export default node;
