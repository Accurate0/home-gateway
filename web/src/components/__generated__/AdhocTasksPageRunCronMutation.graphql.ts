/**
 * @generated SignedSource<<dffdf98cc268a8facfcc253e6e52513b>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AdhocTasksPageRunCronMutation$variables = {
  name: string;
};
export type AdhocTasksPageRunCronMutation$data = {
  readonly runAdhocCronTask: boolean;
};
export type AdhocTasksPageRunCronMutation = {
  response: AdhocTasksPageRunCronMutation$data;
  variables: AdhocTasksPageRunCronMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "name"
  }
],
v1 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Variable",
        "name": "name",
        "variableName": "name"
      }
    ],
    "kind": "ScalarField",
    "name": "runAdhocCronTask",
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "AdhocTasksPageRunCronMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "AdhocTasksPageRunCronMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "c65a8dce40d8cc1714abb5ebb4133f80",
    "id": null,
    "metadata": {},
    "name": "AdhocTasksPageRunCronMutation",
    "operationKind": "mutation",
    "text": "mutation AdhocTasksPageRunCronMutation(\n  $name: String!\n) {\n  runAdhocCronTask(name: $name)\n}\n"
  }
};
})();

(node as any).hash = "04499c50252143305e2501662fb97801";

export default node;
