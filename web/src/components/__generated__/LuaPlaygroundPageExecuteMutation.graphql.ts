/**
 * @generated SignedSource<<ee6bf49ab9ebe67a94c4d38baabdf779>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type LuaPlaygroundPageExecuteMutation$variables = {
  dryRun?: boolean | null | undefined;
  script: string;
};
export type LuaPlaygroundPageExecuteMutation$data = {
  readonly homeExecuteLua: any;
};
export type LuaPlaygroundPageExecuteMutation = {
  response: LuaPlaygroundPageExecuteMutation$data;
  variables: LuaPlaygroundPageExecuteMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "dryRun"
},
v1 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "script"
},
v2 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Variable",
        "name": "dryRun",
        "variableName": "dryRun"
      },
      {
        "kind": "Variable",
        "name": "script",
        "variableName": "script"
      }
    ],
    "kind": "ScalarField",
    "name": "homeExecuteLua",
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
    "name": "LuaPlaygroundPageExecuteMutation",
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
    "name": "LuaPlaygroundPageExecuteMutation",
    "selections": (v2/*:: as any*/)
  },
  "params": {
    "cacheID": "4a79448a5baacb54663b7271f4fe3c49",
    "id": null,
    "metadata": {},
    "name": "LuaPlaygroundPageExecuteMutation",
    "operationKind": "mutation",
    "text": "mutation LuaPlaygroundPageExecuteMutation(\n  $script: String!\n  $dryRun: Boolean\n) {\n  homeExecuteLua(script: $script, dryRun: $dryRun)\n}\n"
  }
};
})();

(node as any).hash = "0a772cc277f77e99c4170f9e5910aaa8";

export default node;
