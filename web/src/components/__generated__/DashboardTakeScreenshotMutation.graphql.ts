/**
 * @generated SignedSource<<9ad3e869366be59fcdbd32fe2eafd725>>
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
  readonly homeEinkDisplay: {
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
    "concreteType": "HomeEinkDisplayMutation",
    "kind": "LinkedField",
    "name": "homeEinkDisplay",
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
    "cacheID": "d064ad8f24af8a41fed8bc9c12e70413",
    "id": null,
    "metadata": {},
    "name": "DashboardTakeScreenshotMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardTakeScreenshotMutation(\n  $id: IdOrAlias!\n) {\n  homeEinkDisplay(id: $id) {\n    takeScreenshot\n  }\n}\n"
  }
};
})();

(node as any).hash = "ce71512061274c563785dfecaf675887";

export default node;
