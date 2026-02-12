/**
 * @generated SignedSource<<637db8f3f6dd707a52668d26c5105300>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type AppQuery$variables = {
  location: string;
  since: any;
};
export type AppQuery$data = {
  readonly environment: {
    readonly outdoor: {
      readonly humidity: number;
      readonly temperature: number;
    };
  };
  readonly solar: {
    readonly current: {
      readonly currentProductionWh: number;
      readonly statistics: {
        readonly averages: {
          readonly last15Mins: number | null | undefined;
          readonly last1Hour: number | null | undefined;
        };
      };
      readonly todayProductionKwh: number;
      readonly uvLevel: number | null | undefined;
    };
    readonly " $fragmentSpreads": FragmentRefs<"SolarChart_solar">;
  };
  readonly weather: {
    readonly " $fragmentSpreads": FragmentRefs<"ForecastCard_weather">;
  };
  readonly woolworths: {
    readonly products: ReadonlyArray<{
      readonly name: string;
      readonly price: number;
    }>;
  };
};
export type AppQuery = {
  response: AppQuery$data;
  variables: AppQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "location"
  },
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "since"
  }
],
v1 = [
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
v2 = {
  "alias": null,
  "args": null,
  "concreteType": "WoolworthsObject",
  "kind": "LinkedField",
  "name": "woolworths",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": null,
      "concreteType": "WoolworthsProducts",
      "kind": "LinkedField",
      "name": "products",
      "plural": true,
      "selections": [
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "name",
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "price",
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "storageKey": null
},
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "uvLevel",
  "storageKey": null
},
v4 = {
  "alias": null,
  "args": null,
  "concreteType": "SolarCurrentResponse",
  "kind": "LinkedField",
  "name": "current",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "todayProductionKwh",
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "currentProductionWh",
      "storageKey": null
    },
    (v3/*: any*/),
    {
      "alias": null,
      "args": null,
      "concreteType": "SolarCurrentStatistics",
      "kind": "LinkedField",
      "name": "statistics",
      "plural": false,
      "selections": [
        {
          "alias": null,
          "args": null,
          "concreteType": "SolarCurrentStatisticsAverages",
          "kind": "LinkedField",
          "name": "averages",
          "plural": false,
          "selections": [
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "last15Mins",
              "storageKey": null
            },
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "last1Hour",
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
},
v5 = [
  {
    "kind": "Variable",
    "name": "since",
    "variableName": "since"
  }
],
v6 = {
  "alias": null,
  "args": null,
  "concreteType": "EnvironmentObject",
  "kind": "LinkedField",
  "name": "environment",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": null,
      "concreteType": "EnvironmentDetails",
      "kind": "LinkedField",
      "name": "outdoor",
      "plural": false,
      "selections": [
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "temperature",
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "humidity",
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "storageKey": null
};
return {
  "fragment": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "AppQuery",
    "selections": [
      {
        "alias": null,
        "args": (v1/*: any*/),
        "concreteType": "WeatherObject",
        "kind": "LinkedField",
        "name": "weather",
        "plural": false,
        "selections": [
          {
            "args": null,
            "kind": "FragmentSpread",
            "name": "ForecastCard_weather"
          }
        ],
        "storageKey": null
      },
      (v2/*: any*/),
      {
        "alias": null,
        "args": null,
        "concreteType": "SolarObject",
        "kind": "LinkedField",
        "name": "solar",
        "plural": false,
        "selections": [
          (v4/*: any*/),
          {
            "args": (v5/*: any*/),
            "kind": "FragmentSpread",
            "name": "SolarChart_solar"
          }
        ],
        "storageKey": null
      },
      (v6/*: any*/)
    ],
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Operation",
    "name": "AppQuery",
    "selections": [
      {
        "alias": null,
        "args": (v1/*: any*/),
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
      },
      (v2/*: any*/),
      {
        "alias": null,
        "args": null,
        "concreteType": "SolarObject",
        "kind": "LinkedField",
        "name": "solar",
        "plural": false,
        "selections": [
          (v4/*: any*/),
          {
            "alias": null,
            "args": [
              {
                "fields": (v5/*: any*/),
                "kind": "ObjectValue",
                "name": "input"
              }
            ],
            "concreteType": "GenerationHistory",
            "kind": "LinkedField",
            "name": "history",
            "plural": true,
            "selections": [
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "wh",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "at",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "timestamp",
                "storageKey": null
              },
              (v3/*: any*/)
            ],
            "storageKey": null
          }
        ],
        "storageKey": null
      },
      (v6/*: any*/)
    ]
  },
  "params": {
    "cacheID": "a6898ca52a62084c44fe3cddbd650d41",
    "id": null,
    "metadata": {},
    "name": "AppQuery",
    "operationKind": "query",
    "text": "query AppQuery(\n  $location: String!\n  $since: DateTime!\n) {\n  weather(input: {location: $location}) {\n    ...ForecastCard_weather\n  }\n  woolworths {\n    products {\n      name\n      price\n    }\n  }\n  solar {\n    current {\n      todayProductionKwh\n      currentProductionWh\n      uvLevel\n      statistics {\n        averages {\n          last15Mins\n          last1Hour\n        }\n      }\n    }\n    ...SolarChart_solar_2xCj2c\n  }\n  environment {\n    outdoor {\n      temperature\n      humidity\n    }\n  }\n}\n\nfragment ForecastCard_weather on WeatherObject {\n  forecast {\n    days {\n      dateTime\n      code\n      description\n      emoji\n      min\n      max\n      uv\n    }\n  }\n}\n\nfragment SolarChart_solar_2xCj2c on SolarObject {\n  history(input: {since: $since}) {\n    wh\n    at\n    timestamp\n    uvLevel\n  }\n}\n"
  }
};
})();

(node as any).hash = "38ec654bdcd876350914127e92e32a96";

export default node;
