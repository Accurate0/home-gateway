/**
 * @generated SignedSource<<2e4f3505d10df5ffc7d3bec22c2abdaf>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type CommandOutcome = "SENT" | "UNCHANGED" | "%future added value";
export type DashboardAirPurifierSetSpeedMutation$variables = {
  id: string;
  speed: number;
};
export type DashboardAirPurifierSetSpeedMutation$data = {
  readonly airPurifier: {
    readonly setSpeed: CommandOutcome;
  };
};
export type DashboardAirPurifierSetSpeedMutation = {
  response: DashboardAirPurifierSetSpeedMutation$data;
  variables: DashboardAirPurifierSetSpeedMutation$variables;
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
    "name": "speed"
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
            "name": "speed",
            "variableName": "speed"
          }
        ],
        "kind": "ScalarField",
        "name": "setSpeed",
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
    "name": "DashboardAirPurifierSetSpeedMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardAirPurifierSetSpeedMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "6be23c45c9d8604ecf7c72146c6bfb04",
    "id": null,
    "metadata": {},
    "name": "DashboardAirPurifierSetSpeedMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardAirPurifierSetSpeedMutation(\n  $id: IdOrAlias!\n  $speed: Int!\n) {\n  airPurifier(id: $id) {\n    setSpeed(speed: $speed)\n  }\n}\n"
  }
};
})();

(node as any).hash = "ffe46835531307022e92e259e96f04c9";

export default node;
