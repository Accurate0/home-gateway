/**
 * @generated SignedSource<<8c9271bbbeaa725608b05a5a800b2fa7>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeAssistantPageQuery$variables = Record<PropertyKey, never>;
export type HomeAssistantPageQuery$data = {
  readonly homeAssistant: {
    readonly entities: ReadonlyArray<{
      readonly entityId: string;
      readonly eventId: any;
      readonly id: string;
      readonly state: string;
      readonly time: any;
    }>;
  };
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
    "concreteType": "HomeAssistantObject",
    "kind": "LinkedField",
    "name": "homeAssistant",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "HomeAssistantEvent",
        "kind": "LinkedField",
        "name": "entities",
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
    "cacheID": "5f027317b9aa6d8f3c2dffef86176a15",
    "id": null,
    "metadata": {},
    "name": "HomeAssistantPageQuery",
    "operationKind": "query",
    "text": "query HomeAssistantPageQuery {\n  homeAssistant {\n    entities {\n      id\n      eventId\n      entityId\n      state\n      time\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "9bf17dc61fa0afbd79998fac4ddea39a";

export default node;
