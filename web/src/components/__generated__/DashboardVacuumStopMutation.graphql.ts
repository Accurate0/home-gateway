/**
 * @generated SignedSource<<bbcaac1ad811cdffdd7af4cf29de079e>>
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
    "cacheID": "cf1f339c35abdee188c7cde4f5a4be93",
    "id": null,
    "metadata": {},
    "name": "DashboardVacuumStopMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardVacuumStopMutation(\n  $id: IdOrAlias!\n) {\n  robotVacuum(id: $id) {\n    stop\n  }\n}\n"
  }
};
})();

(node as any).hash = "f3e2b6d41302a3f059b182abe718a335";

export default node;
