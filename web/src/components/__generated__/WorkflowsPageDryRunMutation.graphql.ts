/**
 * @generated SignedSource<<0c1b90e86d23305da5dfe16194f62b15>>
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
  readonly runWorkflow: any;
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
    "name": "runWorkflow",
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
    "cacheID": "948282717f1fe239020aed6e21d21020",
    "id": null,
    "metadata": {},
    "name": "WorkflowsPageDryRunMutation",
    "operationKind": "mutation",
    "text": "mutation WorkflowsPageDryRunMutation(\n  $slug: String!\n) {\n  runWorkflow(slug: $slug, dryRun: true)\n}\n"
  }
};
})();

(node as any).hash = "9e7389d2abdc28006467ff7a4e71791e";

export default node;
