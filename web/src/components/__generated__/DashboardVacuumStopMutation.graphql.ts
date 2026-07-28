/**
 * @generated SignedSource<<58ca84ab30bc7a89c37bf8f2517f844c>>
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
  readonly robotVacuum: {
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
    "concreteType": "RobotVacuumMutation",
    "kind": "LinkedField",
    "name": "robotVacuum",
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
    "type": "MutationRoot",
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
    "cacheID": "1ea7e286705530df8f1cf92e0439608e",
    "id": null,
    "metadata": {},
    "name": "DashboardVacuumStopMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardVacuumStopMutation(\n  $id: String!\n) {\n  robotVacuum(id: $id) {\n    stop\n  }\n}\n"
  }
};
})();

(node as any).hash = "93aa318c98cbd1c19e9c08c72d42a7cd";

export default node;
