/**
 * @generated SignedSource<<31f890a0d17655afc866bdc772f99bc9>>
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
  readonly homeWorkflowRuns: ReadonlyArray<{
    readonly id: string;
    readonly steps: ReadonlyArray<{
      readonly depth: number;
      readonly detail: string | null | undefined;
      readonly durationUs: number;
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
        "name": "trigger",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "HomeWorkflowRunStep",
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
            "name": "durationUs",
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
    "type": "Query",
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
    "cacheID": "3705ee799a9e9b0aa21aac2de37ba7d8",
    "id": null,
    "metadata": {},
    "name": "RunTraceQuery",
    "operationKind": "query",
    "text": "query RunTraceQuery(\n  $eventId: UUID!\n) {\n  homeWorkflowRuns(eventId: $eventId) {\n    id\n    trigger\n    steps {\n      seq\n      depth\n      kind\n      outcome\n      guard\n      detail\n      error\n      durationUs\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "63671c2ca50aab870b3616443bdea1ba";

export default node;
