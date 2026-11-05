/**
 * @generated SignedSource<<7bce2eb09f3dde25e19c17ac8e9d2939>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardTakeScreenshotMutation$variables = {
  id: string;
};
export type DashboardTakeScreenshotMutation$data = {
  readonly einkDisplay: {
    readonly takeScreenshot: boolean;
  };
};
export type DashboardTakeScreenshotMutation = {
  response: DashboardTakeScreenshotMutation$data;
  variables: DashboardTakeScreenshotMutation$variables;
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
    "concreteType": "EinkDisplayMutation",
    "kind": "LinkedField",
    "name": "einkDisplay",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "takeScreenshot",
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
    "name": "DashboardTakeScreenshotMutation",
    "selections": (v1/*:: as any*/),
    "type": "Mutation",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardTakeScreenshotMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "ea8d0cb22f0b37f0e15af1762ba5e90c",
    "id": null,
    "metadata": {},
    "name": "DashboardTakeScreenshotMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardTakeScreenshotMutation(\n  $id: IdOrAlias!\n) {\n  einkDisplay(id: $id) {\n    takeScreenshot\n  }\n}\n"
  }
};
})();

(node as any).hash = "003416817e0f566950fe064314b64252";

export default node;
