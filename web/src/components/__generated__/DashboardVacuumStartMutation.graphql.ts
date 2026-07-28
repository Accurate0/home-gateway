/**
 * @generated SignedSource<<76a62dfb973ebde5a17fb37d11cc8d63>>
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
    "cacheID": "50c05a4971b7bbf67544e31b5cb21432",
    "id": null,
    "metadata": {},
    "name": "DashboardVacuumStartMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardVacuumStartMutation(\n  $id: String!\n) {\n  robotVacuum(id: $id) {\n    start\n  }\n}\n"
  }
};
})();

(node as any).hash = "3a689097ef4b44fbf0179f58d8f9fff1";

export default node;
