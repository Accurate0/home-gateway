/**
 * @generated SignedSource<<b53eab08089971649f26ba2d3580ab4b>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AdhocTasksPageQuery$variables = Record<PropertyKey, never>;
export type AdhocTasksPageQuery$data = {
  readonly adhocCronTasks: ReadonlyArray<{
    readonly durationMs: number | null | undefined;
    readonly flag: string | null | undefined;
    readonly id: string;
    readonly lastRunAt: any | null | undefined;
    readonly name: string;
    readonly nextRunAt: any | null | undefined;
    readonly outcome: string | null | undefined;
    readonly rowsAffected: number | null | undefined;
    readonly schedule: string;
  }>;
  readonly adhocTasks: ReadonlyArray<{
    readonly checksumDrifted: boolean;
    readonly completedAt: any | null | undefined;
    readonly durationMs: number | null | undefined;
    readonly flag: string | null | undefined;
    readonly id: string;
    readonly name: string;
    readonly ordinal: number;
    readonly pending: boolean;
  }>;
};
export type AdhocTasksPageQuery = {
  response: AdhocTasksPageQuery$data;
  variables: AdhocTasksPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v1 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "flag",
  "storageKey": null
},
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "durationMs",
  "storageKey": null
},
v4 = [
  {
    "alias": null,
    "args": null,
    "concreteType": "AdhocCronTaskStatus",
    "kind": "LinkedField",
    "name": "adhocCronTasks",
    "plural": true,
    "selections": [
      (v0/*:: as any*/),
      (v1/*:: as any*/),
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "schedule",
        "storageKey": null
      },
      (v2/*:: as any*/),
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "nextRunAt",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "lastRunAt",
        "storageKey": null
      },
      (v3/*:: as any*/),
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "rowsAffected",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "outcome",
        "storageKey": null
      }
    ],
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "concreteType": "AdhocTaskStatus",
    "kind": "LinkedField",
    "name": "adhocTasks",
    "plural": true,
    "selections": [
      (v0/*:: as any*/),
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "ordinal",
        "storageKey": null
      },
      (v1/*:: as any*/),
      (v2/*:: as any*/),
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "completedAt",
        "storageKey": null
      },
      (v3/*:: as any*/),
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "pending",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "checksumDrifted",
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
    "name": "AdhocTasksPageQuery",
    "selections": (v4/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "AdhocTasksPageQuery",
    "selections": (v4/*:: as any*/)
  },
  "params": {
    "cacheID": "fd84fb275e71b4d0c62e628652505738",
    "id": null,
    "metadata": {},
    "name": "AdhocTasksPageQuery",
    "operationKind": "query",
    "text": "query AdhocTasksPageQuery {\n  adhocCronTasks {\n    id\n    name\n    schedule\n    flag\n    nextRunAt\n    lastRunAt\n    durationMs\n    rowsAffected\n    outcome\n  }\n  adhocTasks {\n    id\n    ordinal\n    name\n    flag\n    completedAt\n    durationMs\n    pending\n    checksumDrifted\n  }\n}\n"
  }
};
})();

(node as any).hash = "b4038c669359beb488fb853d43fddb2c";

export default node;
