/**
 * @generated SignedSource<<f38397ee9774ef766a75a32f096b7db6>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardMediaStopMutation$variables = {
  id: string;
};
export type DashboardMediaStopMutation$data = {
  readonly homeMediaPlayer: {
    readonly stop: boolean;
  };
};
export type DashboardMediaStopMutation = {
  response: DashboardMediaStopMutation$data;
  variables: DashboardMediaStopMutation$variables;
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
    "concreteType": "HomeMediaPlayerMutation",
    "kind": "LinkedField",
    "name": "homeMediaPlayer",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "stop",
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
    "name": "DashboardMediaStopMutation",
    "selections": (v1/*:: as any*/),
    "type": "Mutation",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardMediaStopMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "da4b8fd915e91cca1f8e6989baf71d94",
    "id": null,
    "metadata": {},
    "name": "DashboardMediaStopMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardMediaStopMutation(\n  $id: IdOrAlias!\n) {\n  homeMediaPlayer(id: $id) {\n    stop\n  }\n}\n"
  }
};
})();

(node as any).hash = "96233dba9f336d890a1fdbe8a18958d0";

export default node;
