/**
 * @generated SignedSource<<7c0bfb14f181bb39f3d41d60b3813725>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type ForecastCardQuery$variables = {
  location: string;
};
export type ForecastCardQuery$data = {
  readonly weather: {
    readonly forecast: {
      readonly days: ReadonlyArray<{
        readonly code: string;
        readonly dateTime: string;
        readonly description: string;
        readonly emoji: string;
        readonly max: number;
        readonly min: number;
        readonly uv: number | null | undefined;
      }>;
    };
  };
};
export type ForecastCardQuery = {
  response: ForecastCardQuery$data;
  variables: ForecastCardQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "location"
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
            "name": "location",
            "variableName": "location"
          }
        ],
        "kind": "ObjectValue",
        "name": "input"
      }
    ],
    "concreteType": "WeatherObject",
    "kind": "LinkedField",
    "name": "weather",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "Forecast",
        "kind": "LinkedField",
        "name": "forecast",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "ForecastDetails",
            "kind": "LinkedField",
            "name": "days",
            "plural": true,
            "selections": [
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "dateTime",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "code",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "description",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "emoji",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "min",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "max",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "uv",
                "storageKey": null
              }
            ],
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
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "ForecastCardQuery",
    "selections": (v1/*: any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Operation",
    "name": "ForecastCardQuery",
    "selections": (v1/*: any*/)
  },
  "params": {
    "cacheID": "079ddbd3f0d504596c3e1b6d075b0958",
    "id": null,
    "metadata": {},
    "name": "ForecastCardQuery",
    "operationKind": "query",
    "text": "query ForecastCardQuery(\n  $location: String!\n) {\n  weather(input: {location: $location}) {\n    forecast {\n      days {\n        dateTime\n        code\n        description\n        emoji\n        min\n        max\n        uv\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "0f9acf35a9327b4dc921503de4c4e0f0";

export default node;
