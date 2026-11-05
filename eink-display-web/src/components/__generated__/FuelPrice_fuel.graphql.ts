/**
 * @generated SignedSource<<1d313cbb6ffacf104a9978ee58021bbe>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type FuelPrice_fuel$data = {
  readonly cheapest: {
    readonly name: string;
    readonly price: number;
    readonly suburb: string;
  } | null | undefined;
  readonly " $fragmentType": "FuelPrice_fuel";
};
export type FuelPrice_fuel$key = {
  readonly " $data"?: FuelPrice_fuel$data;
  readonly " $fragmentSpreads": FragmentRefs<"FuelPrice_fuel">;
};

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": null,
  "name": "FuelPrice_fuel",
  "selections": [
    {
      "alias": null,
      "args": null,
      "concreteType": "HomeFuelSite",
      "kind": "LinkedField",
      "name": "cheapest",
      "plural": false,
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
  "type": "HomeFuelWatchObject",
  "abstractKey": null
};

(node as any).hash = "6032e995d52fb7ae9da6ea79c68f537f";

export default node;
