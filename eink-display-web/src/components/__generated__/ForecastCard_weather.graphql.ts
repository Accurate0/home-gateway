/**
 * @generated SignedSource<<d9dffd1a745d8d82350460b8d09498ff>>
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
  "type": "HomeWeatherObject",
  "abstractKey": null
};

(node as any).hash = "01bd53701abc7c5fd827ba18b394dde2";

export default node;
