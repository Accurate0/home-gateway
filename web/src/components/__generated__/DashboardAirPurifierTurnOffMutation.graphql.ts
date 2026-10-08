/**
 * @generated SignedSource<<a2422e6d50c51acc875fecfda97140be>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type CommandOutcome = "SENT" | "UNCHANGED" | "%future added value";
export type DashboardAirPurifierTurnOffMutation$variables = {
  id: string;
};
export type DashboardAirPurifierTurnOffMutation$data = {
  readonly airPurifier: {
    readonly turnOff: CommandOutcome;
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
    "concreteType": "AirPurifierMutation",
    "kind": "LinkedField",
    "name": "airPurifier",
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
    "type": "MutationRoot",
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
    "cacheID": "03ab4d00a92bab67a57f6606d8c4e61b",
    "id": null,
    "metadata": {},
    "name": "DashboardAirPurifierTurnOffMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardAirPurifierTurnOffMutation(\n  $id: IdOrAlias!\n) {\n  airPurifier(id: $id) {\n    turnOff\n  }\n}\n"
  }
};
})();

(node as any).hash = "e4cd956dc8d0999253257ca1ec53e8cc";

export default node;
