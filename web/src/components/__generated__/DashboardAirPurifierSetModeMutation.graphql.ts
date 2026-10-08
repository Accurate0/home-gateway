/**
 * @generated SignedSource<<66e832ecaab9b17e0efb523670b721d2>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type AirPurifierMode = "AUTO" | "MANUAL" | "SLEEP" | "%future added value";
export type CommandOutcome = "SENT" | "UNCHANGED" | "%future added value";
export type DashboardAirPurifierSetModeMutation$variables = {
  id: string;
  mode: AirPurifierMode;
};
export type DashboardAirPurifierSetModeMutation$data = {
  readonly airPurifier: {
    readonly setMode: CommandOutcome;
  };
};
export type DashboardAirPurifierSetModeMutation = {
  response: DashboardAirPurifierSetModeMutation$data;
  variables: DashboardAirPurifierSetModeMutation$variables;
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
    "name": "mode"
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
            "name": "mode",
            "variableName": "mode"
          }
        ],
        "kind": "ScalarField",
        "name": "setMode",
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
    "name": "DashboardAirPurifierSetModeMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardAirPurifierSetModeMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "628df5b95a243e98ed86bf13cf72c0a7",
    "id": null,
    "metadata": {},
    "name": "DashboardAirPurifierSetModeMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardAirPurifierSetModeMutation(\n  $id: IdOrAlias!\n  $mode: AirPurifierMode!\n) {\n  airPurifier(id: $id) {\n    setMode(mode: $mode)\n  }\n}\n"
  }
};
})();

(node as any).hash = "3d1ca04470cf1fa5b0aecd33b92ec84b";

export default node;
