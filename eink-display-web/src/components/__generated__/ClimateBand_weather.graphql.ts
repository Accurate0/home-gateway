/**
 * @generated SignedSource<<8c58b236421dbdb2b5dd26dce4ce84ef>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type ClimateBand_weather$data = {
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
  readonly " $fragmentType": "ClimateBand_weather";
};
export type ClimateBand_weather$key = {
  readonly " $data"?: ClimateBand_weather$data;
  readonly " $fragmentSpreads": FragmentRefs<"ClimateBand_weather">;
};

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": null,
  "name": "ClimateBand_weather",
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

(node as any).hash = "6709b89741b7e37c6020a21363f841fd";

export default node;
