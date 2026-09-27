/**
 * @generated SignedSource<<3701de64b13eaaa80ac5977e06aec370>>
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
  readonly mediaPlayer: {
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
    "concreteType": "MediaPlayerMutation",
    "kind": "LinkedField",
    "name": "mediaPlayer",
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
    "type": "MutationRoot",
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
    "cacheID": "b2e1707e05876963771a3cf10efa9558",
    "id": null,
    "metadata": {},
    "name": "DashboardMediaStopMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardMediaStopMutation(\n  $id: IdOrAlias!\n) {\n  mediaPlayer(id: $id) {\n    stop\n  }\n}\n"
  }
};
})();

(node as any).hash = "fa889cad8d030b7bdeb80272b4417a7f";

export default node;
