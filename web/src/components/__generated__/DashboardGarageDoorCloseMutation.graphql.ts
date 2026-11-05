/**
 * @generated SignedSource<<6dfd1140165e9b00245c3cf923981ceb>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeCommandOutcome = "LOCKED" | "SENT" | "UNCHANGED" | "%future added value";
export type DashboardGarageDoorCloseMutation$variables = {
  id: string;
};
export type DashboardGarageDoorCloseMutation$data = {
  readonly homeGarageDoor: {
    readonly close: HomeCommandOutcome;
  };
};
export type DashboardGarageDoorCloseMutation = {
  response: DashboardGarageDoorCloseMutation$data;
  variables: DashboardGarageDoorCloseMutation$variables;
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
    "concreteType": "HomeGarageDoorMutation",
    "kind": "LinkedField",
    "name": "homeGarageDoor",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "close",
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
    "name": "DashboardGarageDoorCloseMutation",
    "selections": (v1/*:: as any*/),
    "type": "Mutation",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardGarageDoorCloseMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "0b50726bd4b186e96096ec01d81eaf0c",
    "id": null,
    "metadata": {},
    "name": "DashboardGarageDoorCloseMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardGarageDoorCloseMutation(\n  $id: IdOrAlias!\n) {\n  homeGarageDoor(id: $id) {\n    close\n  }\n}\n"
  }
};
})();

(node as any).hash = "ef841f5a112f8ce976ea95cd8228fd18";

export default node;
