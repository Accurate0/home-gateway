/**
 * @generated SignedSource<<09fb04026f2c052191828300a37e34a8>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardVacuumStartMutation$variables = {
  id: string;
};
export type DashboardVacuumStartMutation$data = {
  readonly homeRobotVacuum: {
    readonly start: boolean;
  };
};
export type DashboardVacuumStartMutation = {
  response: DashboardVacuumStartMutation$data;
  variables: DashboardVacuumStartMutation$variables;
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
    "name": "DashboardVacuumStartMutation",
    "selections": (v1/*:: as any*/),
    "type": "Mutation",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardVacuumStartMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "7af1291494734fe60a3635bf96d07e86",
    "id": null,
    "metadata": {},
    "name": "DashboardVacuumStartMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardVacuumStartMutation(\n  $id: IdOrAlias!\n) {\n  homeRobotVacuum(id: $id) {\n    start\n  }\n}\n"
  }
};
})();

(node as any).hash = "9d1664a7620dedfc829ca41f8cec4f20";

export default node;
