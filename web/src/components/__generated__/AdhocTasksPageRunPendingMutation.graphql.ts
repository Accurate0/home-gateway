/**
 * @generated SignedSource<<9e9551ee4f233145f85eaa1886d00f0d>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AdhocTasksPageRunPendingMutation$variables = Record<PropertyKey, never>;
export type AdhocTasksPageRunPendingMutation$data = {
  readonly homeRunPendingAdhocTasks: boolean;
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
    "name": "homeRunPendingAdhocTasks",
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
    "type": "Mutation",
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
    "cacheID": "6f824ca5a8077c94be264c9f86e3a0bb",
    "id": null,
    "metadata": {},
    "name": "AdhocTasksPageRunPendingMutation",
    "operationKind": "mutation",
    "text": "mutation AdhocTasksPageRunPendingMutation {\n  homeRunPendingAdhocTasks\n}\n"
  }
};
})();

(node as any).hash = "7d385c9be82572096323b5231b9bbef9";

export default node;
