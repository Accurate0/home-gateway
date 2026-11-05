/**
 * @generated SignedSource<<883eb96725b454ec5e1029144796a2a4>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeAssistantPageQuery$variables = Record<PropertyKey, never>;
export type HomeAssistantPageQuery$data = {
  readonly homeHomeAssistant: {
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
    "concreteType": "HomeHomeAssistantObject",
    "kind": "LinkedField",
    "name": "homeHomeAssistant",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "HomeHomeAssistantEvent",
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
    "type": "Query",
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
    "cacheID": "f86f10e839d0017d30409997df6bcc07",
    "id": null,
    "metadata": {},
    "name": "HomeAssistantPageQuery",
    "operationKind": "query",
    "text": "query HomeAssistantPageQuery {\n  homeHomeAssistant {\n    entities {\n      id\n      eventId\n      entityId\n      state\n      time\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "e6fc7240b3a42ae45c86a909f5876073";

export default node;
