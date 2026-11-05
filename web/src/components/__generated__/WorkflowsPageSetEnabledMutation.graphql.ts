/**
 * @generated SignedSource<<e936153e6386051453c41fbfac38d309>>
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
  readonly homeSetWorkflowEnabled: boolean;
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
    "name": "homeSetWorkflowEnabled",
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
    "name": "WorkflowsPageSetEnabledMutation",
    "selections": (v2/*:: as any*/)
  },
  "params": {
    "cacheID": "81c522fb6151e7d844afce64fab8a483",
    "id": null,
    "metadata": {},
    "name": "WorkflowsPageSetEnabledMutation",
    "operationKind": "mutation",
    "text": "mutation WorkflowsPageSetEnabledMutation(\n  $slug: String!\n  $enabled: Boolean!\n) {\n  homeSetWorkflowEnabled(slug: $slug, enabled: $enabled)\n}\n"
  }
};
})();

(node as any).hash = "bb64f6d7830f343439b00220e94dd175";

export default node;
