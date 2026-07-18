/**
 * @generated SignedSource<<dc3a1d01bfb4e785875fc3bb417381b6>>
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
    "cacheID": "e7898022da0be04beb62c5bde95b675d",
    "id": null,
    "metadata": {},
    "name": "HomeAssistantPageQuery",
    "operationKind": "query",
    "text": "query HomeAssistantPageQuery(\n  $since: DateTime!\n) {\n  events(input: {since: $since}) {\n    homeAssistant {\n      eventId\n      entityId\n      state\n      time\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "c9c4a2817b418187b5ed931ac0ede759";

export default node;
