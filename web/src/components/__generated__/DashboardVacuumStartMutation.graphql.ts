/**
 * @generated SignedSource<<b6bd7f58e5256d6360f66ec852c6881f>>
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
  readonly robotVacuum: {
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
    "concreteType": "RobotVacuumMutation",
    "kind": "LinkedField",
    "name": "robotVacuum",
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
    "type": "MutationRoot",
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
    "cacheID": "67cdd8ceec75496ebebe5c1db8d18f40",
    "id": null,
    "metadata": {},
    "name": "DashboardVacuumStartMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardVacuumStartMutation(\n  $id: IdOrAlias!\n) {\n  robotVacuum(id: $id) {\n    start\n  }\n}\n"
  }
};
})();

(node as any).hash = "76a5823910d109c08b2d96bb1fabdeb9";

export default node;
