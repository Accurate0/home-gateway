/**
 * @generated SignedSource<<d0e7be19447d72d2374243c8e0ab0d31>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type WorkflowsPageSetEnabledMutation$variables = {
  enabled: boolean;
  slug: string;
};
export type WorkflowsPageSetEnabledMutation$data = {
  readonly setWorkflowEnabled: boolean;
};
export type WorkflowsPageSetEnabledMutation = {
  response: WorkflowsPageSetEnabledMutation$data;
  variables: WorkflowsPageSetEnabledMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "enabled"
},
v1 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "slug"
},
v2 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Variable",
        "name": "enabled",
        "variableName": "enabled"
      },
      {
        "kind": "Variable",
        "name": "slug",
        "variableName": "slug"
      }
    ],
    "kind": "ScalarField",
    "name": "setWorkflowEnabled",
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
    "name": "WorkflowsPageSetEnabledMutation",
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
    "name": "WorkflowsPageSetEnabledMutation",
    "selections": (v2/*:: as any*/)
  },
  "params": {
    "cacheID": "334c2181b1b8b36ac69b06871aa7e83e",
    "id": null,
    "metadata": {},
    "name": "WorkflowsPageSetEnabledMutation",
    "operationKind": "mutation",
    "text": "mutation WorkflowsPageSetEnabledMutation(\n  $slug: String!\n  $enabled: Boolean!\n) {\n  setWorkflowEnabled(slug: $slug, enabled: $enabled)\n}\n"
  }
};
})();

(node as any).hash = "aa0c03f0fa8b468559f46b64c2b19bca";

export default node;
