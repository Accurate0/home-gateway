/**
 * @generated SignedSource<<54d246b896dd61a282841e23d58dbd35>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type Mode = "AWAY" | "GUEST" | "HOME" | "NIGHT" | "PARTY" | "VACATION" | "%future added value";
export type AwayPageSetModeMutation$variables = {
  active: boolean;
};
export type AwayPageSetModeMutation$data = {
  readonly setMode: ReadonlyArray<Mode>;
};
export type AwayPageSetModeMutation = {
  response: AwayPageSetModeMutation$data;
  variables: AwayPageSetModeMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "active"
  }
],
v1 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Variable",
        "name": "active",
        "variableName": "active"
      },
      {
        "kind": "Literal",
        "name": "mode",
        "value": "AWAY"
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
    "name": "AwayPageSetModeMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "AwayPageSetModeMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "c4d3ce31bf3bcbac3fe2d6060776c989",
    "id": null,
    "metadata": {},
    "name": "AwayPageSetModeMutation",
    "operationKind": "mutation",
    "text": "mutation AwayPageSetModeMutation(\n  $active: Boolean!\n) {\n  setMode(mode: AWAY, active: $active)\n}\n"
  }
};
})();

(node as any).hash = "7a8fc4e484d1e6ab90f2008d38e65e23";

export default node;
