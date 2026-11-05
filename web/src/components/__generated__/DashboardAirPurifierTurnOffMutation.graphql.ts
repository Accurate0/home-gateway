/**
 * @generated SignedSource<<965ddf12f54a9f82c0638d270346d605>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeCommandOutcome = "LOCKED" | "SENT" | "UNCHANGED" | "%future added value";
export type DashboardAirPurifierTurnOffMutation$variables = {
  id: string;
};
export type DashboardAirPurifierTurnOffMutation$data = {
  readonly homeAirPurifier: {
    readonly turnOff: HomeCommandOutcome;
  };
};
export type DashboardAirPurifierTurnOffMutation = {
  response: DashboardAirPurifierTurnOffMutation$data;
  variables: DashboardAirPurifierTurnOffMutation$variables;
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
    "concreteType": "HomeAirPurifierMutation",
    "kind": "LinkedField",
    "name": "homeAirPurifier",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "turnOff",
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
    "name": "DashboardAirPurifierTurnOffMutation",
    "selections": (v1/*:: as any*/),
    "type": "Mutation",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardAirPurifierTurnOffMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "7e2e82aa8e90bc8543215443972b7a5f",
    "id": null,
    "metadata": {},
    "name": "DashboardAirPurifierTurnOffMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardAirPurifierTurnOffMutation(\n  $id: IdOrAlias!\n) {\n  homeAirPurifier(id: $id) {\n    turnOff\n  }\n}\n"
  }
};
})();

(node as any).hash = "a88663eb59b3cf94efdc3066a6011370";

export default node;
