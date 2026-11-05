/**
 * @generated SignedSource<<3253639fb97685bb0290e45b06aff44a>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardVacuumStopMutation$variables = {
  id: string;
};
export type DashboardVacuumStopMutation$data = {
  readonly homeRobotVacuum: {
    readonly stop: boolean;
  };
};
export type DashboardVacuumStopMutation = {
  response: DashboardVacuumStopMutation$data;
  variables: DashboardVacuumStopMutation$variables;
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
        "name": "stop",
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
    "name": "DashboardVacuumStopMutation",
    "selections": (v1/*:: as any*/),
    "type": "Mutation",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardVacuumStopMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "2492cb0756a44633496f03d26e888603",
    "id": null,
    "metadata": {},
    "name": "DashboardVacuumStopMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardVacuumStopMutation(\n  $id: IdOrAlias!\n) {\n  homeRobotVacuum(id: $id) {\n    stop\n  }\n}\n"
  }
};
})();

(node as any).hash = "a840f51705ce58a4f2c1732061255552";

export default node;
