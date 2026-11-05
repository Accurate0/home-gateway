/**
 * @generated SignedSource<<2aa148cb05104d2905ec59b41d7307ab>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardMediaPlayPauseMutation$variables = {
  id: string;
};
export type DashboardMediaPlayPauseMutation$data = {
  readonly homeMediaPlayer: {
    readonly playPause: boolean;
  };
};
export type DashboardMediaPlayPauseMutation = {
  response: DashboardMediaPlayPauseMutation$data;
  variables: DashboardMediaPlayPauseMutation$variables;
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
        "name": "playPause",
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
    "name": "DashboardMediaPlayPauseMutation",
    "selections": (v1/*:: as any*/),
    "type": "Mutation",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardMediaPlayPauseMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "793a830eec3eaaaa026ce222e131ab87",
    "id": null,
    "metadata": {},
    "name": "DashboardMediaPlayPauseMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardMediaPlayPauseMutation(\n  $id: IdOrAlias!\n) {\n  homeMediaPlayer(id: $id) {\n    playPause\n  }\n}\n"
  }
};
})();

(node as any).hash = "01d08a1c351e2aede7324385d04c14a7";

export default node;
