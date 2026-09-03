/**
 * @generated SignedSource<<6ac5cc00d8b689360be3053634eb6051>>
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
      "concreteType": "FuelSite",
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
  "type": "FuelWatchObject",
  "abstractKey": null
};

(node as any).hash = "e9cfea03ac2fd4951db74659fb2f3aa8";

export default node;
