/**
 * @generated SignedSource<<3fc987bbc1dec32cb50d774c218a5f39>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type WorkflowsPageDryRunMutation$variables = {
  slug: string;
};
export type WorkflowsPageDryRunMutation$data = {
  readonly homeRunWorkflow: any;
};
export type WorkflowsPageDryRunMutation = {
  response: WorkflowsPageDryRunMutation$data;
  variables: WorkflowsPageDryRunMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "slug"
  }
],
v1 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Literal",
        "name": "dryRun",
        "value": true
      },
      {
        "kind": "Variable",
        "name": "slug",
        "variableName": "slug"
      }
    ],
    "kind": "ScalarField",
    "name": "homeRunWorkflow",
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "WorkflowsPageDryRunMutation",
    "selections": (v1/*:: as any*/),
    "type": "Mutation",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "WorkflowsPageDryRunMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "c6c21d682d1cd089a48e4ef286ef449b",
    "id": null,
    "metadata": {},
    "name": "WorkflowsPageDryRunMutation",
    "operationKind": "mutation",
    "text": "mutation WorkflowsPageDryRunMutation(\n  $slug: String!\n) {\n  homeRunWorkflow(slug: $slug, dryRun: true)\n}\n"
  }
};
})();

(node as any).hash = "75e15439b5185234ea5dbdeef2471d73";

export default node;
