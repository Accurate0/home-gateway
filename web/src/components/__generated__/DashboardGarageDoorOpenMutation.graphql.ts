/**
 * @generated SignedSource<<d1ab319ab878a3d13d965f4180725f71>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type CommandOutcome = "SENT" | "UNCHANGED" | "%future added value";
export type DashboardGarageDoorOpenMutation$variables = {
  id: string;
};
export type DashboardGarageDoorOpenMutation$data = {
  readonly garageDoor: {
    readonly open: CommandOutcome;
  };
};
export type DashboardGarageDoorOpenMutation = {
  response: DashboardGarageDoorOpenMutation$data;
  variables: DashboardGarageDoorOpenMutation$variables;
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
        "name": "open",
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
    "name": "DashboardGarageDoorOpenMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardGarageDoorOpenMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "1bd97e2ab95b40023178a26ecb605283",
    "id": null,
    "metadata": {},
    "name": "DashboardGarageDoorOpenMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardGarageDoorOpenMutation(\n  $id: IdOrAlias!\n) {\n  garageDoor(id: $id) {\n    open\n  }\n}\n"
  }
};
})();

(node as any).hash = "8573bed2ee5db64d36b96a25984c0470";

export default node;
