/**
 * @generated SignedSource<<73975584aae046b44eafc97efac0d2da>>
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
    "cacheID": "6fc71d3e1bd1e77eb6c6c31c13c1123e",
    "id": null,
    "metadata": {},
    "name": "DashboardSetOnMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetOnMutation(\n  $id: IdOrAlias!\n) {\n  light(id: $id) {\n    on\n  }\n}\n"
  }
};
})();

(node as any).hash = "77ae4baead144d7ec1091b50ac957c58";

export default node;
