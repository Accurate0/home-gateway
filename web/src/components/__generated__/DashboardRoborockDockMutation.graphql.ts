/**
 * @generated SignedSource<<7bea87d0a8dd2d48503f20a317d55de7>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardRoborockDockMutation$variables = {
  id: string;
};
export type DashboardRoborockDockMutation$data = {
  readonly roborock: {
    readonly dock: boolean;
  };
};
export type DashboardRoborockDockMutation = {
  response: DashboardRoborockDockMutation$data;
  variables: DashboardRoborockDockMutation$variables;
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
    "name": "DashboardRoborockDockMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardRoborockDockMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "29b73e636edeb5fb951227d691d47348",
    "id": null,
    "metadata": {},
    "name": "DashboardRoborockDockMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardRoborockDockMutation(\n  $id: String!\n) {\n  roborock(id: $id) {\n    dock\n  }\n}\n"
  }
};
})();

(node as any).hash = "6ed63dee06151f561d8acfd74c4dd594";

export default node;
