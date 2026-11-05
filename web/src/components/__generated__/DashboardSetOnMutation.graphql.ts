/**
 * @generated SignedSource<<f62c5b7649f207149c9e7004c1ecba25>>
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
  readonly homeLight: {
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
    "concreteType": "HomeLightMutation",
    "kind": "LinkedField",
    "name": "homeLight",
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
    "type": "Mutation",
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
    "cacheID": "0da39b27f21362330bafb65966c9be56",
    "id": null,
    "metadata": {},
    "name": "DashboardSetOnMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetOnMutation(\n  $id: IdOrAlias!\n) {\n  homeLight(id: $id) {\n    on\n  }\n}\n"
  }
};
})();

(node as any).hash = "c4fdc3b43dc1bae0895129bb82832883";

export default node;
