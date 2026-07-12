/**
 * @generated SignedSource<<243b1035e70257a299adb62fab644557>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardSetOnMutation$variables = {
  id: string;
};
export type DashboardSetOnMutation$data = {
  readonly light: {
    readonly on: boolean;
  };
};
export type DashboardSetOnMutation = {
  response: DashboardSetOnMutation$data;
  variables: DashboardSetOnMutation$variables;
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
    "concreteType": "LightMutation",
    "kind": "LinkedField",
    "name": "light",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "on",
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
    "name": "DashboardSetOnMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardSetOnMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "274c9ee8d09255f6fbbff672930a60fd",
    "id": null,
    "metadata": {},
    "name": "DashboardSetOnMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetOnMutation(\n  $id: String!\n) {\n  light(id: $id) {\n    on\n  }\n}\n"
  }
};
})();

(node as any).hash = "bc07693f3805a73fabfaf4627616d397";

export default node;
