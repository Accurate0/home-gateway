/**
 * @generated SignedSource<<c3e07e0ba74cee0180f862ce40a78e06>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeCommandOutcome = "LOCKED" | "SENT" | "UNCHANGED" | "%future added value";
export type DashboardAirPurifierTurnOnMutation$variables = {
  id: string;
};
export type DashboardAirPurifierTurnOnMutation$data = {
  readonly homeAirPurifier: {
    readonly turnOn: HomeCommandOutcome;
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
    "concreteType": "HomeAirPurifierMutation",
    "kind": "LinkedField",
    "name": "homeAirPurifier",
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
    "type": "Mutation",
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
    "cacheID": "533c19f4496581f7f2c321381ece6bfb",
    "id": null,
    "metadata": {},
    "name": "DashboardAirPurifierTurnOnMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardAirPurifierTurnOnMutation(\n  $id: IdOrAlias!\n) {\n  homeAirPurifier(id: $id) {\n    turnOn\n  }\n}\n"
  }
};
})();

(node as any).hash = "30329d3f2bccbcf43ca00797bed50f6e";

export default node;
