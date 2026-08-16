/**
 * @generated SignedSource<<eb42b198aaa5dbb854278435ed78b0a6>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AdhocTasksPageRunPendingMutation$variables = Record<PropertyKey, never>;
export type AdhocTasksPageRunPendingMutation$data = {
  readonly runPendingAdhocTasks: boolean;
};
export type AdhocTasksPageRunPendingMutation = {
  response: AdhocTasksPageRunPendingMutation$data;
  variables: AdhocTasksPageRunPendingMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "runPendingAdhocTasks",
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "AdhocTasksPageRunPendingMutation",
    "selections": (v0/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "AdhocTasksPageRunPendingMutation",
    "selections": (v0/*:: as any*/)
  },
  "params": {
    "cacheID": "0ccfa6b342a398ac31318e5d3ef5bd01",
    "id": null,
    "metadata": {},
    "name": "AdhocTasksPageRunPendingMutation",
    "operationKind": "mutation",
    "text": "mutation AdhocTasksPageRunPendingMutation {\n  runPendingAdhocTasks\n}\n"
  }
};
})();

(node as any).hash = "706fae935f3da27e016f837b39a0e6cd";

export default node;
