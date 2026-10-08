/**
 * @generated SignedSource<<3730c2fb2c4893d916ea895cce9b175f>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type CommandOutcome = "SENT" | "UNCHANGED" | "%future added value";
export type DashboardAirPurifierSetDisplayMutation$variables = {
  id: string;
  on: boolean;
};
export type DashboardAirPurifierSetDisplayMutation$data = {
  readonly airPurifier: {
    readonly setDisplay: CommandOutcome;
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
    "concreteType": "AirPurifierMutation",
    "kind": "LinkedField",
    "name": "airPurifier",
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
    "type": "MutationRoot",
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
    "cacheID": "1a74c4e86162ec8e0b61c844e812e3cc",
    "id": null,
    "metadata": {},
    "name": "DashboardAirPurifierSetDisplayMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardAirPurifierSetDisplayMutation(\n  $id: IdOrAlias!\n  $on: Boolean!\n) {\n  airPurifier(id: $id) {\n    setDisplay(on: $on)\n  }\n}\n"
  }
};
})();

(node as any).hash = "129b531e4d9140ae48ce5effc91dc339";

export default node;
