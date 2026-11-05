/**
 * @generated SignedSource<<6d3f9f7413c8b380a3fd3e467cbfc807>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeAirPurifierMode = "AUTO" | "MANUAL" | "SLEEP" | "%future added value";
export type HomeCommandOutcome = "LOCKED" | "SENT" | "UNCHANGED" | "%future added value";
export type DashboardAirPurifierSetModeMutation$variables = {
  id: string;
  mode: HomeAirPurifierMode;
};
export type DashboardAirPurifierSetModeMutation$data = {
  readonly homeAirPurifier: {
    readonly setMode: HomeCommandOutcome;
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
    "type": "Mutation",
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
    "cacheID": "787002babbf5926364d8d7e2735e2830",
    "id": null,
    "metadata": {},
    "name": "DashboardAirPurifierSetModeMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardAirPurifierSetModeMutation(\n  $id: IdOrAlias!\n  $mode: HomeAirPurifierMode!\n) {\n  homeAirPurifier(id: $id) {\n    setMode(mode: $mode)\n  }\n}\n"
  }
};
})();

(node as any).hash = "95c2cc75c9ab7a3bda78f574b7bbf404";

export default node;
