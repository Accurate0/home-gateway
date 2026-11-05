/**
 * @generated SignedSource<<28b5201d161eece530c1b40a2a4f7e44>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeCommandOutcome = "LOCKED" | "SENT" | "UNCHANGED" | "%future added value";
export type DashboardGarageDoorOpenMutation$variables = {
  id: string;
};
export type DashboardGarageDoorOpenMutation$data = {
  readonly homeGarageDoor: {
    readonly open: HomeCommandOutcome;
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
    "concreteType": "HomeGarageDoorMutation",
    "kind": "LinkedField",
    "name": "homeGarageDoor",
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
    "type": "Mutation",
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
    "cacheID": "ddbe85ae80451f17ce155e9470cdeafc",
    "id": null,
    "metadata": {},
    "name": "DashboardGarageDoorOpenMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardGarageDoorOpenMutation(\n  $id: IdOrAlias!\n) {\n  homeGarageDoor(id: $id) {\n    open\n  }\n}\n"
  }
};
})();

(node as any).hash = "b8f7b60ead519cde754101c8fc81e97b";

export default node;
