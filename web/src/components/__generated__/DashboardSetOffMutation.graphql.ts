/**
 * @generated SignedSource<<e5eed6f1a1aba7b425946d39590587bf>>
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
  readonly homeLight: {
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
    "concreteType": "HomeLightMutation",
    "kind": "LinkedField",
    "name": "homeLight",
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
    "type": "Mutation",
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
    "cacheID": "1bdf4de22ec018813a7db136c28c1938",
    "id": null,
    "metadata": {},
    "name": "DashboardSetOffMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetOffMutation(\n  $id: IdOrAlias!\n) {\n  homeLight(id: $id) {\n    off\n  }\n}\n"
  }
};
})();

(node as any).hash = "b0c602005baf56daae0cdfca482671b5";

export default node;
