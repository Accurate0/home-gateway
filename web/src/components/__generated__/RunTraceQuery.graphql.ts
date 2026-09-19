/**
 * @generated SignedSource<<d41144f44cf1849910edd60a6eecd616>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type RunTraceQuery$variables = {
  eventId: any;
};
export type RunTraceQuery$data = {
  readonly workflowRuns: ReadonlyArray<{
    readonly id: string;
    readonly steps: ReadonlyArray<{
      readonly depth: number;
      readonly detail: string | null | undefined;
      readonly durationMs: number;
      readonly error: string | null | undefined;
      readonly guard: string | null | undefined;
      readonly kind: string;
      readonly outcome: string;
      readonly seq: number;
    }>;
    readonly trigger: any | null | undefined;
  }>;
};
export type RunTraceQuery = {
  response: RunTraceQuery$data;
  variables: RunTraceQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "eventId"
  }
],
v1 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Variable",
        "name": "eventId",
        "variableName": "eventId"
      }
    ],
    "concreteType": "WorkflowRun",
    "kind": "LinkedField",
    "name": "workflowRuns",
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
        "name": "trigger",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "WorkflowRunStep",
        "kind": "LinkedField",
        "name": "steps",
        "plural": true,
        "selections": [
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "seq",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "depth",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "kind",
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
            "name": "guard",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "detail",
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
            "name": "durationMs",
            "storageKey": null
          }
        ],
        "storageKey": null
      }
    ],
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "RunTraceQuery",
    "selections": (v1/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "RunTraceQuery",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "4e31032c0177ab77052c03bd7b21f839",
    "id": null,
    "metadata": {},
    "name": "RunTraceQuery",
    "operationKind": "query",
    "text": "query RunTraceQuery(\n  $eventId: UUID!\n) {\n  workflowRuns(eventId: $eventId) {\n    id\n    trigger\n    steps {\n      seq\n      depth\n      kind\n      outcome\n      guard\n      detail\n      error\n      durationMs\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "b6cc1047cd1f7000795639ce58e10fc6";

export default node;
