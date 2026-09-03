/**
 * @generated SignedSource<<d493942b63788ab8935a5f8b32edf2c6>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type AppQuery$variables = {
  location: string;
  since: any;
  transitRoute: string;
};
export type AppQuery$data = {
  readonly fuelwatch: {
    readonly " $fragmentSpreads": FragmentRefs<"FuelPrice_fuel">;
  };
  readonly hallwayPanel: {
    readonly batteryPercentage: number | null | undefined;
    readonly name: string;
  };
  readonly livingRoomPanel: {
    readonly batteryPercentage: number | null | undefined;
    readonly name: string;
  };
  readonly outdoor: {
    readonly humidity: number | null | undefined;
    readonly temperature: number | null | undefined;
  };
  readonly solar: {
    readonly current: {
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
  readonly transperth: {
    readonly route: {
      readonly " $fragmentSpreads": FragmentRefs<"NextTrainTile_route">;
    } | null | undefined;
  };
  readonly weather: {
    readonly " $fragmentSpreads": FragmentRefs<"ClimateBand_weather" | "ForecastCard_weather">;
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
  },
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "transitRoute"
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
  "kind": "ScalarField",
  "name": "uvLevel",
  "storageKey": null
},
v3 = {
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
    (v2/*:: as any*/),
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
v4 = [
  {
    "kind": "Variable",
    "name": "since",
    "variableName": "since"
  }
],
v5 = [
  {
    "kind": "Variable",
    "name": "id",
    "variableName": "transitRoute"
  }
],
v6 = {
  "alias": "outdoor",
  "args": [
    {
      "kind": "Literal",
      "name": "id",
      "value": "env-outdoor"
    }
  ],
  "concreteType": "EnvironmentEntity",
  "kind": "LinkedField",
  "name": "environment",
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
  "storageKey": "environment(id:\"env-outdoor\")"
},
v7 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v8 = [
  (v7/*:: as any*/),
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "batteryPercentage",
    "storageKey": null
  }
],
v9 = {
  "alias": "hallwayPanel",
  "args": [
    {
      "kind": "Literal",
      "name": "id",
      "value": "hallway-epd"
    }
  ],
  "concreteType": "EinkDisplayEntity",
  "kind": "LinkedField",
  "name": "einkDisplay",
  "plural": false,
  "selections": (v8/*:: as any*/),
  "storageKey": "einkDisplay(id:\"hallway-epd\")"
},
v10 = {
  "alias": "livingRoomPanel",
  "args": [
    {
      "kind": "Literal",
      "name": "id",
      "value": "living-room-epd"
    }
  ],
  "concreteType": "EinkDisplayEntity",
  "kind": "LinkedField",
  "name": "einkDisplay",
  "plural": false,
  "selections": (v8/*:: as any*/),
  "storageKey": "einkDisplay(id:\"living-room-epd\")"
};
return {
  "fragment": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "AppQuery",
    "selections": [
      {
        "alias": null,
        "args": (v1/*:: as any*/),
        "concreteType": "WeatherObject",
        "kind": "LinkedField",
        "name": "weather",
        "plural": false,
        "selections": [
          {
            "args": null,
            "kind": "FragmentSpread",
            "name": "ClimateBand_weather"
          },
          {
            "args": null,
            "kind": "FragmentSpread",
            "name": "ForecastCard_weather"
          }
        ],
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "SolarObject",
        "kind": "LinkedField",
        "name": "solar",
        "plural": false,
        "selections": [
          (v3/*:: as any*/),
          {
            "args": (v4/*:: as any*/),
            "kind": "FragmentSpread",
            "name": "SolarChart_solar"
          }
        ],
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "TransperthObject",
        "kind": "LinkedField",
        "name": "transperth",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": (v5/*:: as any*/),
            "concreteType": "RouteDeparturesObject",
            "kind": "LinkedField",
            "name": "route",
            "plural": false,
            "selections": [
              {
                "args": null,
                "kind": "FragmentSpread",
                "name": "NextTrainTile_route"
              }
            ],
            "storageKey": null
          }
        ],
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "FuelWatchObject",
        "kind": "LinkedField",
        "name": "fuelwatch",
        "plural": false,
        "selections": [
          {
            "args": null,
            "kind": "FragmentSpread",
            "name": "FuelPrice_fuel"
          }
        ],
        "storageKey": null
      },
      (v6/*:: as any*/),
      (v9/*:: as any*/),
      (v10/*:: as any*/)
    ],
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "AppQuery",
    "selections": [
      {
        "alias": null,
        "args": (v1/*:: as any*/),
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
      {
        "alias": null,
        "args": null,
        "concreteType": "SolarObject",
        "kind": "LinkedField",
        "name": "solar",
        "plural": false,
        "selections": [
          (v3/*:: as any*/),
          {
            "alias": null,
            "args": [
              {
                "fields": (v4/*:: as any*/),
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
              (v2/*:: as any*/)
            ],
            "storageKey": null
          }
        ],
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "TransperthObject",
        "kind": "LinkedField",
        "name": "transperth",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": (v5/*:: as any*/),
            "concreteType": "RouteDeparturesObject",
            "kind": "LinkedField",
            "name": "route",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "origin",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "destination",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "concreteType": "DepartureObject",
                "kind": "LinkedField",
                "name": "departures",
                "plural": true,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "line",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "platform",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "scheduledDeparture",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "delayMinutes",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "minutesAway",
                    "storageKey": null
                  },
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "live",
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
      {
        "alias": null,
        "args": null,
        "concreteType": "FuelWatchObject",
        "kind": "LinkedField",
        "name": "fuelwatch",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "FuelSite",
            "kind": "LinkedField",
            "name": "cheapest",
            "plural": false,
            "selections": [
              (v7/*:: as any*/),
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "suburb",
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
      (v6/*:: as any*/),
      (v9/*:: as any*/),
      (v10/*:: as any*/)
    ]
  },
  "params": {
    "cacheID": "c1dbe546986eace888dd84a879e7ce9b",
    "id": null,
    "metadata": {},
    "name": "AppQuery",
    "operationKind": "query",
    "text": "query AppQuery(\n  $location: String!\n  $since: DateTime!\n  $transitRoute: String!\n) {\n  weather(input: {location: $location}) {\n    ...ClimateBand_weather\n    ...ForecastCard_weather\n  }\n  solar {\n    current {\n      todayProductionKwh\n      uvLevel\n      statistics {\n        averages {\n          last15Mins\n          last1Hour\n        }\n      }\n    }\n    ...SolarChart_solar_2xCj2c\n  }\n  transperth {\n    route(id: $transitRoute) {\n      ...NextTrainTile_route\n    }\n  }\n  fuelwatch {\n    ...FuelPrice_fuel\n  }\n  outdoor: environment(id: \"env-outdoor\") {\n    temperature\n    humidity\n  }\n  hallwayPanel: einkDisplay(id: \"hallway-epd\") {\n    name\n    batteryPercentage\n  }\n  livingRoomPanel: einkDisplay(id: \"living-room-epd\") {\n    name\n    batteryPercentage\n  }\n}\n\nfragment ClimateBand_weather on WeatherObject {\n  forecast {\n    days {\n      dateTime\n      code\n      description\n      min\n      max\n      uv\n    }\n  }\n}\n\nfragment ForecastCard_weather on WeatherObject {\n  forecast {\n    days {\n      dateTime\n      code\n      description\n      min\n      max\n      uv\n    }\n  }\n}\n\nfragment FuelPrice_fuel on FuelWatchObject {\n  cheapest {\n    name\n    suburb\n    price\n  }\n}\n\nfragment NextTrainTile_route on RouteDeparturesObject {\n  origin\n  destination\n  departures {\n    line\n    platform\n    scheduledDeparture\n    delayMinutes\n    minutesAway\n    live\n  }\n}\n\nfragment SolarChart_solar_2xCj2c on SolarObject {\n  history(input: {since: $since}) {\n    wh\n    at\n    timestamp\n    uvLevel\n  }\n}\n"
  }
};
})();

(node as any).hash = "0c629f0a777440857f6d4f22c09db48c";

export default node;
