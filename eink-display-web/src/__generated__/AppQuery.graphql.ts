/**
 * @generated SignedSource<<00e8830b9294f307af7e0720dafae6d8>>
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
  readonly hallwayPanel: {
    readonly batteryPercentage: number | null | undefined;
    readonly name: string;
  };
  readonly homeFuelwatch: {
    readonly " $fragmentSpreads": FragmentRefs<"FuelPrice_fuel">;
  };
  readonly homeSolar: {
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
  readonly homeTransperth: {
    readonly route: {
      readonly " $fragmentSpreads": FragmentRefs<"TransitFuelRow_route">;
    } | null | undefined;
  };
  readonly homeWeather: {
    readonly " $fragmentSpreads": FragmentRefs<"ClimateBand_weather" | "ForecastCard_weather">;
  };
  readonly livingRoomPanel: {
    readonly batteryPercentage: number | null | undefined;
    readonly name: string;
  };
  readonly outdoor: {
    readonly humidity: number | null | undefined;
    readonly temperature: number | null | undefined;
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
  "concreteType": "HomeSolarCurrentResponse",
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
      "concreteType": "HomeSolarCurrentStatistics",
      "kind": "LinkedField",
      "name": "statistics",
      "plural": false,
      "selections": [
        {
          "alias": null,
          "args": null,
          "concreteType": "HomeSolarCurrentStatisticsAverages",
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
  "concreteType": "HomeEnvironmentEntity",
  "kind": "LinkedField",
  "name": "homeEnvironment",
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
  "storageKey": "homeEnvironment(id:\"env-outdoor\")"
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
  "concreteType": "HomeEinkDisplayEntity",
  "kind": "LinkedField",
  "name": "homeEinkDisplay",
  "plural": false,
  "selections": (v8/*:: as any*/),
  "storageKey": "homeEinkDisplay(id:\"hallway-epd\")"
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
  "concreteType": "HomeEinkDisplayEntity",
  "kind": "LinkedField",
  "name": "homeEinkDisplay",
  "plural": false,
  "selections": (v8/*:: as any*/),
  "storageKey": "homeEinkDisplay(id:\"living-room-epd\")"
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
        "concreteType": "HomeWeatherObject",
        "kind": "LinkedField",
        "name": "homeWeather",
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
        "concreteType": "HomeSolarObject",
        "kind": "LinkedField",
        "name": "homeSolar",
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
        "concreteType": "HomeTransperthObject",
        "kind": "LinkedField",
        "name": "homeTransperth",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": (v5/*:: as any*/),
            "concreteType": "HomeRouteDeparturesObject",
            "kind": "LinkedField",
            "name": "route",
            "plural": false,
            "selections": [
              {
                "args": null,
                "kind": "FragmentSpread",
                "name": "TransitFuelRow_route"
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
        "concreteType": "HomeFuelWatchObject",
        "kind": "LinkedField",
        "name": "homeFuelwatch",
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
    "type": "Query",
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
        "concreteType": "HomeWeatherObject",
        "kind": "LinkedField",
        "name": "homeWeather",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "HomeForecast",
            "kind": "LinkedField",
            "name": "forecast",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": "HomeForecastDetails",
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
        "concreteType": "HomeSolarObject",
        "kind": "LinkedField",
        "name": "homeSolar",
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
            "concreteType": "HomeGenerationHistory",
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
        "concreteType": "HomeTransperthObject",
        "kind": "LinkedField",
        "name": "homeTransperth",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": (v5/*:: as any*/),
            "concreteType": "HomeRouteDeparturesObject",
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
                "concreteType": "HomeDepartureObject",
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
        "concreteType": "HomeFuelWatchObject",
        "kind": "LinkedField",
        "name": "homeFuelwatch",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "concreteType": "HomeFuelSite",
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
    "cacheID": "7938cc5e2255dff433963ce5ce7a6370",
    "id": null,
    "metadata": {},
    "name": "AppQuery",
    "operationKind": "query",
    "text": "query AppQuery(\n  $location: String!\n  $since: DateTime!\n  $transitRoute: String!\n) {\n  homeWeather(input: {location: $location}) {\n    ...ClimateBand_weather\n    ...ForecastCard_weather\n  }\n  homeSolar {\n    current {\n      todayProductionKwh\n      uvLevel\n      statistics {\n        averages {\n          last15Mins\n          last1Hour\n        }\n      }\n    }\n    ...SolarChart_solar_2xCj2c\n  }\n  homeTransperth {\n    route(id: $transitRoute) {\n      ...TransitFuelRow_route\n    }\n  }\n  homeFuelwatch {\n    ...FuelPrice_fuel\n  }\n  outdoor: homeEnvironment(id: \"env-outdoor\") {\n    temperature\n    humidity\n  }\n  hallwayPanel: homeEinkDisplay(id: \"hallway-epd\") {\n    name\n    batteryPercentage\n  }\n  livingRoomPanel: homeEinkDisplay(id: \"living-room-epd\") {\n    name\n    batteryPercentage\n  }\n}\n\nfragment ClimateBand_weather on HomeWeatherObject {\n  forecast {\n    days {\n      dateTime\n      code\n      description\n      min\n      max\n      uv\n    }\n  }\n}\n\nfragment ForecastCard_weather on HomeWeatherObject {\n  forecast {\n    days {\n      dateTime\n      code\n      description\n      min\n      max\n      uv\n    }\n  }\n}\n\nfragment FuelPrice_fuel on HomeFuelWatchObject {\n  cheapest {\n    name\n    suburb\n    price\n  }\n}\n\nfragment SolarChart_solar_2xCj2c on HomeSolarObject {\n  history(input: {since: $since}) {\n    wh\n    at\n    timestamp\n    uvLevel\n  }\n}\n\nfragment TransitFuelRow_route on HomeRouteDeparturesObject {\n  origin\n  destination\n  departures {\n    line\n    platform\n    scheduledDeparture\n    delayMinutes\n    minutesAway\n    live\n  }\n}\n"
  }
};
})();

(node as any).hash = "33c17ae1db20e3639800112c4a5c818d";

export default node;
