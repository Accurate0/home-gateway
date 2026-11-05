/**
 * @generated SignedSource<<00b947d3374491c397a1d058897b6944>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeCommandOutcome = "LOCKED" | "SENT" | "UNCHANGED" | "%future added value";
export type DashboardAirPurifierSetDisplayMutation$variables = {
  id: string;
  on: boolean;
};
export type DashboardAirPurifierSetDisplayMutation$data = {
  readonly homeAirPurifier: {
    readonly setDisplay: HomeCommandOutcome;
  };
};
export type DashboardAirPurifierSetDisplayMutation = {
  response: DashboardAirPurifierSetDisplayMutation$data;
  variables: DashboardAirPurifierSetDisplayMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "id"
  },
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "on"
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
    "concreteType": "HomeAirPurifierMutation",
    "kind": "LinkedField",
    "name": "homeAirPurifier",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": [
          {
            "kind": "Variable",
            "name": "on",
            "variableName": "on"
          }
        ],
        "kind": "ScalarField",
        "name": "setDisplay",
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
    "name": "DashboardAirPurifierSetDisplayMutation",
    "selections": (v1/*:: as any*/),
    "type": "Mutation",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardAirPurifierSetDisplayMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "d7625bab7288f4eae710d3ed9039a935",
    "id": null,
    "metadata": {},
    "name": "DashboardAirPurifierSetDisplayMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardAirPurifierSetDisplayMutation(\n  $id: IdOrAlias!\n  $on: Boolean!\n) {\n  homeAirPurifier(id: $id) {\n    setDisplay(on: $on)\n  }\n}\n"
  }
};
})();

(node as any).hash = "e06804e6e9c7ad62f46c2835db3ea37e";

export default node;
