/**
 * @generated SignedSource<<0528f7e3777548ef84f1cef1ddccb40e>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type TransitFuelRow_route$data = {
  readonly departures: ReadonlyArray<{
    readonly delayMinutes: number | null | undefined;
    readonly line: string;
    readonly live: boolean;
    readonly minutesAway: number;
    readonly platform: string | null | undefined;
    readonly scheduledDeparture: any;
  }>;
  readonly destination: string;
  readonly origin: string;
  readonly " $fragmentType": "TransitFuelRow_route";
};
export type TransitFuelRow_route$key = {
  readonly " $data"?: TransitFuelRow_route$data;
  readonly " $fragmentSpreads": FragmentRefs<"TransitFuelRow_route">;
};

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": null,
  "name": "TransitFuelRow_route",
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
  "type": "HomeRouteDeparturesObject",
  "abstractKey": null
};

(node as any).hash = "bc22bb74bb2f905bfbf7ad9788217ae3";

export default node;
