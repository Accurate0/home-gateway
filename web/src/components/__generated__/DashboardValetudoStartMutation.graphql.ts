/**
 * @generated SignedSource<<9ec483bdd526de9b8ff57d1911407a1f>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardValetudoStartMutation$variables = {
  id: string;
};
export type DashboardValetudoStartMutation$data = {
  readonly valetudo: {
    readonly start: boolean;
  };
};
export type DashboardValetudoStartMutation = {
  response: DashboardValetudoStartMutation$data;
  variables: DashboardValetudoStartMutation$variables;
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
    "name": "DashboardValetudoStartMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardValetudoStartMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "2680bad7e7379c8d76c451ba0cf38f72",
    "id": null,
    "metadata": {},
    "name": "DashboardValetudoStartMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardValetudoStartMutation(\n  $id: String!\n) {\n  valetudo(id: $id) {\n    start\n  }\n}\n"
  }
};
})();

(node as any).hash = "5d4953cdb9db9fb142119ab6bf0376d4";

export default node;
