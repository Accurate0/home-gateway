/**
 * @generated SignedSource<<03a2ee5272e8fd7488c0cd1772c15887>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type WorkflowsPageQuery$variables = Record<PropertyKey, never>;
export type WorkflowsPageQuery$data = {
  readonly workflows: ReadonlyArray<{
    readonly configEnabled: boolean;
    readonly dryRun: boolean;
    readonly enabled: boolean;
    readonly group: string;
    readonly id: string;
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
    "concreteType": "WorkflowStatus",
    "kind": "LinkedField",
    "name": "workflows",
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
    "type": "QueryRoot",
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
    "cacheID": "26cac6d2689bc6c7dc7595d003e505a4",
    "id": null,
    "metadata": {},
    "name": "WorkflowsPageQuery",
    "operationKind": "query",
    "text": "query WorkflowsPageQuery {\n  workflows {\n    id\n    slug\n    name\n    group\n    enabled\n    configEnabled\n    dryRun\n    reusable\n  }\n}\n"
  }
};
})();

(node as any).hash = "b159c040e84c60e37cd27792dfb05168";

export default node;
