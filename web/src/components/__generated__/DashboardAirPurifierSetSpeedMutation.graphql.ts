/**
 * @generated SignedSource<<8f500cdbe085f7b28dbbc2737e8ac353>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeCommandOutcome = "LOCKED" | "SENT" | "UNCHANGED" | "%future added value";
export type DashboardAirPurifierSetSpeedMutation$variables = {
  id: string;
  speed: number;
};
export type DashboardAirPurifierSetSpeedMutation$data = {
  readonly homeAirPurifier: {
    readonly setSpeed: HomeCommandOutcome;
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
    "type": "Mutation",
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
    "cacheID": "9396a6a577cd885876fe0abd4fe93786",
    "id": null,
    "metadata": {},
    "name": "DashboardAirPurifierSetSpeedMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardAirPurifierSetSpeedMutation(\n  $id: IdOrAlias!\n  $speed: Int!\n) {\n  homeAirPurifier(id: $id) {\n    setSpeed(speed: $speed)\n  }\n}\n"
  }
};
})();

(node as any).hash = "b92327ecfdf22a7db5601a149bf915a1";

export default node;
