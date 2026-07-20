/**
 * @generated SignedSource<<999e4b8b01af0e0663d21b51aea2da30>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeAssistantPageQuery$variables = Record<PropertyKey, never>;
export type HomeAssistantPageQuery$data = {
  readonly homeAssistantEntities: ReadonlyArray<{
    readonly entityId: string;
    readonly eventId: any;
    readonly id: string;
    readonly state: string;
    readonly time: any;
  }>;
};
export type HomeAssistantPageQuery = {
  response: HomeAssistantPageQuery$data;
  variables: HomeAssistantPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "alias": null,
    "args": null,
    "concreteType": "HomeAssistantEvent",
    "kind": "LinkedField",
    "name": "homeAssistantEntities",
    "plural": true,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "id",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "eventId",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "entityId",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "state",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "time",
        "storageKey": null
      }
    ],
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "HomeAssistantPageQuery",
    "selections": (v0/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "HomeAssistantPageQuery",
    "selections": (v0/*:: as any*/)
  },
  "params": {
    "cacheID": "85701966556480e6272528ff2bf7f376",
    "id": null,
    "metadata": {},
    "name": "HomeAssistantPageQuery",
    "operationKind": "query",
    "text": "query HomeAssistantPageQuery {\n  homeAssistantEntities {\n    id\n    eventId\n    entityId\n    state\n    time\n  }\n}\n"
  }
};
})();

(node as any).hash = "c35856b8e0a09cef3017fd73a398958b";

export default node;
