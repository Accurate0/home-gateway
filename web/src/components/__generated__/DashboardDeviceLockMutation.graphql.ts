/**
 * @generated SignedSource<<679c6c7dd5461cb5356c9455e9a3b607>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeDeviceLockSource = "MANUAL" | "PHYSICAL" | "%future added value";
export type DashboardDeviceLockMutation$variables = {
  durationSeconds?: number | null | undefined;
  id: string;
};
export type DashboardDeviceLockMutation$data = {
  readonly homeDeviceLock: {
    readonly lock: {
      readonly expiresAt: any | null | undefined;
      readonly source: HomeDeviceLockSource;
    };
  };
};
export type DashboardDeviceLockMutation = {
  response: DashboardDeviceLockMutation$data;
  variables: DashboardDeviceLockMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "durationSeconds"
},
v1 = {
  "defaultValue": null,
  "kind": "LocalArgument",
  "name": "id"
},
v2 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Variable",
        "name": "id",
        "variableName": "id"
      }
    ],
    "concreteType": "HomeDeviceLockMutation",
    "kind": "LinkedField",
    "name": "homeDeviceLock",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": [
          {
            "kind": "Variable",
            "name": "durationSeconds",
            "variableName": "durationSeconds"
          }
        ],
        "concreteType": "HomeDeviceLock",
        "kind": "LinkedField",
        "name": "lock",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "source",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "expiresAt",
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
    "argumentDefinitions": [
      (v0/*:: as any*/),
      (v1/*:: as any*/)
    ],
    "kind": "Fragment",
    "metadata": null,
    "name": "DashboardDeviceLockMutation",
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
    "name": "DashboardDeviceLockMutation",
    "selections": (v2/*:: as any*/)
  },
  "params": {
    "cacheID": "ac4553594e94e90bafacff32c5f1537b",
    "id": null,
    "metadata": {},
    "name": "DashboardDeviceLockMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardDeviceLockMutation(\n  $id: IdOrAlias!\n  $durationSeconds: Int\n) {\n  homeDeviceLock(id: $id) {\n    lock(durationSeconds: $durationSeconds) {\n      source\n      expiresAt\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "cb545e81b3459b44fd33f8296688ee4c";

export default node;
