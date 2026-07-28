/**
 * @generated SignedSource<<4d6a982097ccc5d81f45e6515135f681>>
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
    "type": "MutationRoot",
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
    "cacheID": "bf399f810b2439a07ddba918aa7da861",
    "id": null,
    "metadata": {},
    "name": "DashboardTakeScreenshotMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardTakeScreenshotMutation(\n  $id: String!\n) {\n  einkDisplay(id: $id) {\n    takeScreenshot\n  }\n}\n"
  }
};
})();

(node as any).hash = "6f4e7e37af57fee41f19829e9bea2ce1";

export default node;
