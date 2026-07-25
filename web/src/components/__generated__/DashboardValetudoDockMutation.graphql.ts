/**
 * @generated SignedSource<<81d4aa37bbc18987f4128ee6af6487ab>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardValetudoDockMutation$variables = {
  id: string;
};
export type DashboardValetudoDockMutation$data = {
  readonly valetudo: {
    readonly dock: boolean;
  };
};
export type DashboardValetudoDockMutation = {
  response: DashboardValetudoDockMutation$data;
  variables: DashboardValetudoDockMutation$variables;
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
    "name": "DashboardValetudoDockMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardValetudoDockMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "e469f52298a7aac2241d5250543cb74e",
    "id": null,
    "metadata": {},
    "name": "DashboardValetudoDockMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardValetudoDockMutation(\n  $id: String!\n) {\n  valetudo(id: $id) {\n    dock\n  }\n}\n"
  }
};
})();

(node as any).hash = "db36f18edcc539b359087779d0d68cb8";

export default node;
