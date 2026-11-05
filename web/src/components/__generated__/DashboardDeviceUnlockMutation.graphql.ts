/**
 * @generated SignedSource<<7cee43462656755d28954dd154b558f0>>
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
  readonly homeDeviceLock: {
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
    "concreteType": "HomeDeviceLockMutation",
    "kind": "LinkedField",
    "name": "homeDeviceLock",
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
    "type": "Mutation",
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
    "cacheID": "8af63f57131d2470d1896ecb448eb5ee",
    "id": null,
    "metadata": {},
    "name": "DashboardDeviceUnlockMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardDeviceUnlockMutation(\n  $id: IdOrAlias!\n) {\n  homeDeviceLock(id: $id) {\n    unlock\n  }\n}\n"
  }
};
})();

(node as any).hash = "7c8bab02fe3465dd55b96edf148b26bc";

export default node;
