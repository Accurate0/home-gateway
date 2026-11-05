/**
 * @generated SignedSource<<6e51236ac9ba3a406ca633e607fc2b5e>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeMode = "AWAY" | "GUEST" | "HOME" | "VACATION" | "%future added value";
export type WorkflowsPageQuery$variables = Record<PropertyKey, never>;
export type WorkflowsPageQuery$data = {
  readonly homeWorkflows: ReadonlyArray<{
    readonly configEnabled: boolean;
    readonly dryRun: boolean;
    readonly enabled: boolean;
    readonly group: string;
    readonly id: string;
    readonly modes: ReadonlyArray<HomeMode>;
    readonly name: string;
    readonly reusable: boolean;
    readonly slug: string;
  }>;
};
export type WorkflowsPageQuery = {
  response: WorkflowsPageQuery$data;
  variables: WorkflowsPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "alias": null,
    "args": null,
    "concreteType": "HomeWorkflowStatus",
    "kind": "LinkedField",
    "name": "homeWorkflows",
    "plural": true,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "id",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "slug",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "name",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "group",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "enabled",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "configEnabled",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "dryRun",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "reusable",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "modes",
        "storageKey": null
      }
    ],
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "WorkflowsPageQuery",
    "selections": (v0/*:: as any*/),
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "WorkflowsPageQuery",
    "selections": (v0/*:: as any*/)
  },
  "params": {
    "cacheID": "d0f34f7b6b8abc0c3f92d2959c9dc1a5",
    "id": null,
    "metadata": {},
    "name": "WorkflowsPageQuery",
    "operationKind": "query",
    "text": "query WorkflowsPageQuery {\n  homeWorkflows {\n    id\n    slug\n    name\n    group\n    enabled\n    configEnabled\n    dryRun\n    reusable\n    modes\n  }\n}\n"
  }
};
})();

(node as any).hash = "1ad0a283f38e53381ee4bfba8ea49f35";

export default node;
