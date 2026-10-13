/**
 * @generated SignedSource<<032c74888f784b60467366e40e6e566d>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardDeviceUnlockMutation$variables = {
  id: string;
};
export type DashboardDeviceUnlockMutation$data = {
  readonly deviceLock: {
    readonly unlock: boolean;
  };
};
export type DashboardDeviceUnlockMutation = {
  response: DashboardDeviceUnlockMutation$data;
  variables: DashboardDeviceUnlockMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "id"
  }
],
v1 = [
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
        "args": null,
        "kind": "ScalarField",
        "name": "unlock",
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
    "name": "DashboardDeviceUnlockMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardDeviceUnlockMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "dff0e65e7e4257f1adf477bf3db1bc32",
    "id": null,
    "metadata": {},
    "name": "DashboardDeviceUnlockMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardDeviceUnlockMutation(\n  $id: IdOrAlias!\n) {\n  deviceLock(id: $id) {\n    unlock\n  }\n}\n"
  }
};
})();

(node as any).hash = "56fc1286cfadc31d3afeffac70b22acd";

export default node;
