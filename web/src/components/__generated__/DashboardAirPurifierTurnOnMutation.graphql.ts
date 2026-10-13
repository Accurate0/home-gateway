/**
 * @generated SignedSource<<f0e3fce3779755ad278739bc35c6597e>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type CommandOutcome = "LOCKED" | "SENT" | "UNCHANGED" | "%future added value";
export type DashboardAirPurifierTurnOnMutation$variables = {
  id: string;
};
export type DashboardAirPurifierTurnOnMutation$data = {
  readonly airPurifier: {
    readonly turnOn: CommandOutcome;
  };
};
export type DashboardAirPurifierTurnOnMutation = {
  response: DashboardAirPurifierTurnOnMutation$data;
  variables: DashboardAirPurifierTurnOnMutation$variables;
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
    "concreteType": "AirPurifierMutation",
    "kind": "LinkedField",
    "name": "airPurifier",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "turnOn",
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
    "name": "DashboardAirPurifierTurnOnMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardAirPurifierTurnOnMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "3b5d24638e6521f99813c16c281e1dad",
    "id": null,
    "metadata": {},
    "name": "DashboardAirPurifierTurnOnMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardAirPurifierTurnOnMutation(\n  $id: IdOrAlias!\n) {\n  airPurifier(id: $id) {\n    turnOn\n  }\n}\n"
  }
};
})();

(node as any).hash = "29ac44b77771983f6fa09ba9e7f3a6be";

export default node;
