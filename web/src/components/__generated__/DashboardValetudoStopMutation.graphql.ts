/**
 * @generated SignedSource<<069d995535d2e35facd6fa8109e59507>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardValetudoStopMutation$variables = {
  id: string;
};
export type DashboardValetudoStopMutation$data = {
  readonly valetudo: {
    readonly stop: boolean;
  };
};
export type DashboardValetudoStopMutation = {
  response: DashboardValetudoStopMutation$data;
  variables: DashboardValetudoStopMutation$variables;
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
    "concreteType": "ValetudoMutation",
    "kind": "LinkedField",
    "name": "valetudo",
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
    "name": "DashboardValetudoStopMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardValetudoStopMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "453f89e892e7de70f20692daaa6b6874",
    "id": null,
    "metadata": {},
    "name": "DashboardValetudoStopMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardValetudoStopMutation(\n  $id: String!\n) {\n  valetudo(id: $id) {\n    stop\n  }\n}\n"
  }
};
})();

(node as any).hash = "fcc6a80773d7228b3fe6e682e5a6adb7";

export default node;
