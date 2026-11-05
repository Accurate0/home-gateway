/**
 * @generated SignedSource<<0a3c246840b9f53599dceafa5e581cbb>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type RunsPageQuery$variables = Record<PropertyKey, never>;
export type RunsPageQuery$data = {
  readonly homeWorkflowRuns: ReadonlyArray<{
    readonly dryRun: boolean;
    readonly durationMs: number;
    readonly error: string | null | undefined;
    readonly eventId: string;
    readonly id: string;
    readonly name: string;
    readonly outcome: string;
    readonly slug: string;
    readonly startedAt: any;
  }>;
};
export type RunsPageQuery = {
  response: RunsPageQuery$data;
  variables: RunsPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Literal",
        "name": "limit",
        "value": 100
      }
    ],
    "concreteType": "HomeWorkflowRun",
    "kind": "LinkedField",
    "name": "homeWorkflowRuns",
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
        "name": "eventId",
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
        "name": "outcome",
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
        "name": "durationMs",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "error",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "startedAt",
        "storageKey": null
      }
    ],
    "storageKey": "homeWorkflowRuns(limit:100)"
  }
];
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "RunsPageQuery",
    "selections": (v0/*:: as any*/),
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "RunsPageQuery",
    "selections": (v0/*:: as any*/)
  },
  "params": {
    "cacheID": "49a38f7399722326e7678180211c46a3",
    "id": null,
    "metadata": {},
    "name": "RunsPageQuery",
    "operationKind": "query",
    "text": "query RunsPageQuery {\n  homeWorkflowRuns(limit: 100) {\n    id\n    eventId\n    slug\n    name\n    outcome\n    dryRun\n    durationMs\n    error\n    startedAt\n  }\n}\n"
  }
};
})();

(node as any).hash = "a234b608da8714ab703f34734949379f";

export default node;
