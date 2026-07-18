/**
 * @generated SignedSource<<35e4a4580a46fe5f64cae45332f6eae4>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type HomeAssistantPageQuery$variables = {
  since: any;
};
export type HomeAssistantPageQuery$data = {
  readonly events: {
    readonly homeAssistant: ReadonlyArray<{
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
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "since"
  }
],
v1 = [
  {
    "alias": null,
    "args": [
      {
        "fields": [
          {
            "kind": "Variable",
            "name": "since",
            "variableName": "since"
          }
        ],
        "kind": "ObjectValue",
        "name": "input"
      }
    ],
    "concreteType": "EventsObject",
    "kind": "LinkedField",
    "name": "events",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "HomeAssistantEvent",
        "kind": "LinkedField",
        "name": "homeAssistant",
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
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "HomeAssistantPageQuery",
    "selections": (v1/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "HomeAssistantPageQuery",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "1b3a62d5dbfa4ee01455bf1bbc5dc120",
    "id": null,
    "metadata": {},
    "name": "HomeAssistantPageQuery",
    "operationKind": "query",
    "text": "query HomeAssistantPageQuery(\n  $since: DateTime!\n) {\n  events(input: {since: $since}) {\n    homeAssistant {\n      id\n      eventId\n      entityId\n      state\n      time\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "6a5333506a05f2fba32cf92d7cc82dfd";

export default node;
