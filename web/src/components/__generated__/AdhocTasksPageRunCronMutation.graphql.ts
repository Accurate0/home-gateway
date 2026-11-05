/**
 * @generated SignedSource<<a920883d841965f5d6e9ea715be65397>>
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
  readonly homeRunAdhocCronTask: boolean;
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
    "name": "homeRunAdhocCronTask",
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
    "type": "Mutation",
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
    "cacheID": "ae0acf2e4613c665c9aa160ff581c7c0",
    "id": null,
    "metadata": {},
    "name": "AdhocTasksPageRunCronMutation",
    "operationKind": "mutation",
    "text": "mutation AdhocTasksPageRunCronMutation(\n  $name: String!\n) {\n  homeRunAdhocCronTask(name: $name)\n}\n"
  }
};
})();

(node as any).hash = "e6bc155d1197082d2054bf5825c2128b";

export default node;
