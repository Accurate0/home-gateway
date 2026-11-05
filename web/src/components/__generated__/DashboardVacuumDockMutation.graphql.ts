/**
 * @generated SignedSource<<4cb6edf87233f39b5091931dd9d0cecc>>
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
  readonly homeRobotVacuum: {
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
    "concreteType": "HomeRobotVacuumMutation",
    "kind": "LinkedField",
    "name": "homeRobotVacuum",
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
    "type": "Mutation",
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
    "cacheID": "074e48df2c11ad0d14c717b72149052b",
    "id": null,
    "metadata": {},
    "name": "DashboardVacuumDockMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardVacuumDockMutation(\n  $id: IdOrAlias!\n) {\n  homeRobotVacuum(id: $id) {\n    dock\n  }\n}\n"
  }
};
})();

(node as any).hash = "687ccb0cf1e0803ce394465a35a1b6eb";

export default node;
