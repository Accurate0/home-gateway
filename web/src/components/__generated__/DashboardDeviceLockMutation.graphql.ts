/**
 * @generated SignedSource<<f7548dc22bf48501eca2d567c170eaa0>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DeviceLockSource = "MANUAL" | "PHYSICAL" | "%future added value";
export type DashboardDeviceLockMutation$variables = {
  durationSeconds?: number | null | undefined;
  id: string;
};
export type DashboardDeviceLockMutation$data = {
  readonly deviceLock: {
    readonly lock: {
      readonly expiresAt: any | null | undefined;
      readonly source: DeviceLockSource;
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
    "concreteType": "DeviceLockMutation",
    "kind": "LinkedField",
    "name": "deviceLock",
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
        "concreteType": "DeviceLock",
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
    "name": "DashboardDeviceLockMutation",
    "selections": (v2/*:: as any*/)
  },
  "params": {
    "cacheID": "36f173b229b70e172f9e6c592b72bc81",
    "id": null,
    "metadata": {},
    "name": "DashboardDeviceLockMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardDeviceLockMutation(\n  $id: IdOrAlias!\n  $durationSeconds: Int\n) {\n  deviceLock(id: $id) {\n    lock(durationSeconds: $durationSeconds) {\n      source\n      expiresAt\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "4c17db246168a43f5ae7796850014d83";

export default node;
