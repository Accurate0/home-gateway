/**
 * @generated SignedSource<<f9ae15b15e5f05f696f2791f028a8777>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardSetOffMutation$variables = {
  id: string;
};
export type DashboardSetOffMutation$data = {
  readonly light: {
    readonly off: boolean;
  };
};
export type DashboardSetOffMutation = {
  response: DashboardSetOffMutation$data;
  variables: DashboardSetOffMutation$variables;
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
        "name": "off",
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
    "name": "DashboardSetOffMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardSetOffMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "056f6b57d5458c0c574f5fdffa583578",
    "id": null,
    "metadata": {},
    "name": "DashboardSetOffMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetOffMutation(\n  $id: String!\n) {\n  light(id: $id) {\n    off\n  }\n}\n"
  }
};
})();

(node as any).hash = "8b821fc0ea4432a804b21f4f6b400d11";

export default node;
