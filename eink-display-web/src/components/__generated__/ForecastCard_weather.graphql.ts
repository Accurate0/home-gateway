/**
 * @generated SignedSource<<c5d0775b4de408bb7314546bd858d1fc>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type ForecastCard_weather$data = {
  readonly forecast: {
    readonly days: ReadonlyArray<{
      readonly code: string;
      readonly dateTime: string;
      readonly description: string;
      readonly max: number;
      readonly min: number;
      readonly uv: number | null | undefined;
    }>;
  };
  readonly " $fragmentType": "ForecastCard_weather";
};
export type ForecastCard_weather$key = {
  readonly " $data"?: ForecastCard_weather$data;
  readonly " $fragmentSpreads": FragmentRefs<"ForecastCard_weather">;
};

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": null,
  "name": "ForecastCard_weather",
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
  "type": "WeatherObject",
  "abstractKey": null
};

(node as any).hash = "18a399ca3236bf4a97ad5fe822b71e19";

export default node;
