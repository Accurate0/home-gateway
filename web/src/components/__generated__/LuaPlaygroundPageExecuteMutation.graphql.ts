/**
 * @generated SignedSource<<048d7276530b47e3a2221be53f5e2c42>>
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
  readonly executeLua: any;
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
    "name": "executeLua",
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
    "name": "LuaPlaygroundPageExecuteMutation",
    "selections": (v2/*:: as any*/)
  },
  "params": {
    "cacheID": "af65ff299fc5cf788174d34b52c3b498",
    "id": null,
    "metadata": {},
    "name": "LuaPlaygroundPageExecuteMutation",
    "operationKind": "mutation",
    "text": "mutation LuaPlaygroundPageExecuteMutation(\n  $script: String!\n  $dryRun: Boolean\n) {\n  executeLua(script: $script, dryRun: $dryRun)\n}\n"
  }
};
})();

(node as any).hash = "427bb99f4f4a5082d7f568fa5059aff1";

export default node;
