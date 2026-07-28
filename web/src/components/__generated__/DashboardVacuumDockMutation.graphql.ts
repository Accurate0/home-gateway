/**
 * @generated SignedSource<<0369390177dd2f899610bcb27cff7582>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardVacuumDockMutation$variables = {
  id: string;
};
export type DashboardVacuumDockMutation$data = {
  readonly robotVacuum: {
    readonly dock: boolean;
  };
};
export type DashboardVacuumDockMutation = {
  response: DashboardVacuumDockMutation$data;
  variables: DashboardVacuumDockMutation$variables;
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
    "concreteType": "RobotVacuumMutation",
    "kind": "LinkedField",
    "name": "robotVacuum",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "dock",
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
    "name": "DashboardVacuumDockMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardVacuumDockMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "cc6422dc78ae3384a263b456b19750c9",
    "id": null,
    "metadata": {},
    "name": "DashboardVacuumDockMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardVacuumDockMutation(\n  $id: String!\n) {\n  robotVacuum(id: $id) {\n    dock\n  }\n}\n"
  }
};
})();

(node as any).hash = "77cf4ce407976b6b75b6f97850c846da";

export default node;
