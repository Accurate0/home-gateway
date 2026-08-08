/**
 * @generated SignedSource<<c99cb4708c781c00a9a072241125e3f5>>
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
    "cacheID": "df3ada6583e3f303952ab1c08d6dfcf8",
    "id": null,
    "metadata": {},
    "name": "DashboardMediaPlayPauseMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardMediaPlayPauseMutation(\n  $id: String!\n) {\n  mediaPlayer(id: $id) {\n    playPause\n  }\n}\n"
  }
};
})();

(node as any).hash = "f6864a625e37b90f04aaf2f42e4678e9";

export default node;
