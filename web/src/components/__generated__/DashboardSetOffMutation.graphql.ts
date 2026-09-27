/**
 * @generated SignedSource<<3f914224aab76a2a648cd8b7ea476214>>
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
    "cacheID": "218736ff7ffdecd877f694d6dd4aec0f",
    "id": null,
    "metadata": {},
    "name": "DashboardSetOffMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetOffMutation(\n  $id: IdOrAlias!\n) {\n  light(id: $id) {\n    off\n  }\n}\n"
  }
};
})();

(node as any).hash = "901d9ee0b446a9c7b0927b861ba1b452";

export default node;
