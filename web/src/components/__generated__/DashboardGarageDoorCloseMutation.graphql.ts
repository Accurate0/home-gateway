/**
 * @generated SignedSource<<952ae9cfd350ea3777f686bbfd53b544>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type CommandOutcome = "SENT" | "UNCHANGED" | "%future added value";
export type DashboardGarageDoorCloseMutation$variables = {
  id: string;
};
export type DashboardGarageDoorCloseMutation$data = {
  readonly garageDoor: {
    readonly close: CommandOutcome;
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
    "concreteType": "GarageDoorMutation",
    "kind": "LinkedField",
    "name": "garageDoor",
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
    "type": "MutationRoot",
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
    "cacheID": "2f7290ee43c825796f50403cbfce46ac",
    "id": null,
    "metadata": {},
    "name": "DashboardGarageDoorCloseMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardGarageDoorCloseMutation(\n  $id: IdOrAlias!\n) {\n  garageDoor(id: $id) {\n    close\n  }\n}\n"
  }
};
})();

(node as any).hash = "24552b247c37cc7924b54b47a15ec0ed";

export default node;
