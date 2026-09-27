/**
 * @generated SignedSource<<55a6b4ad3c3c2f40045ea846af023418>>
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
  readonly mediaPlayer: {
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
    "concreteType": "MediaPlayerMutation",
    "kind": "LinkedField",
    "name": "mediaPlayer",
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
    "type": "MutationRoot",
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
    "cacheID": "4b07121857b460e8a9d28e7878b4d979",
    "id": null,
    "metadata": {},
    "name": "DashboardMediaPlayPauseMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardMediaPlayPauseMutation(\n  $id: IdOrAlias!\n) {\n  mediaPlayer(id: $id) {\n    playPause\n  }\n}\n"
  }
};
})();

(node as any).hash = "b09c7c0d42e36e4519de36585f1f9217";

export default node;
