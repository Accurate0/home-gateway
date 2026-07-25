/**
 * @generated SignedSource<<f04469a009557df32e4746edf88f7489>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardRoborockStopMutation$variables = {
  id: string;
};
export type DashboardRoborockStopMutation$data = {
  readonly roborock: {
    readonly stop: boolean;
  };
};
export type DashboardRoborockStopMutation = {
  response: DashboardRoborockStopMutation$data;
  variables: DashboardRoborockStopMutation$variables;
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
    "name": "DashboardRoborockStopMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardRoborockStopMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "4528594c0f753785be65d7a1a8d4251c",
    "id": null,
    "metadata": {},
    "name": "DashboardRoborockStopMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardRoborockStopMutation(\n  $id: String!\n) {\n  roborock(id: $id) {\n    stop\n  }\n}\n"
  }
};
})();

(node as any).hash = "53240bf475fa391741a7ef54161190f4";

export default node;
