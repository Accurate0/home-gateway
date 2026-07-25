/**
 * @generated SignedSource<<96114cccf94c51a5ddc85ca172c9a8e2>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardRoborockStartMutation$variables = {
  id: string;
};
export type DashboardRoborockStartMutation$data = {
  readonly roborock: {
    readonly start: boolean;
  };
};
export type DashboardRoborockStartMutation = {
  response: DashboardRoborockStartMutation$data;
  variables: DashboardRoborockStartMutation$variables;
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
    "concreteType": "RoborockMutation",
    "kind": "LinkedField",
    "name": "roborock",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "start",
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
    "name": "DashboardRoborockStartMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardRoborockStartMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "5f2b8c9f52fcb873829cdb26a8665a2b",
    "id": null,
    "metadata": {},
    "name": "DashboardRoborockStartMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardRoborockStartMutation(\n  $id: String!\n) {\n  roborock(id: $id) {\n    start\n  }\n}\n"
  }
};
})();

(node as any).hash = "e5b0fd64c1a33e2f8d204959492f5921";

export default node;
