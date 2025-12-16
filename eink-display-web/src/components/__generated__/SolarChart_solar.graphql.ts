/**
 * @generated SignedSource<<6380c17ce781237365cd7b25d28b0acf>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type SolarChart_solar$data = {
  readonly history: ReadonlyArray<{
    readonly at: any;
    readonly timestamp: number;
    readonly wh: number;
  }>;
  readonly " $fragmentType": "SolarChart_solar";
};
export type SolarChart_solar$key = {
  readonly " $data"?: SolarChart_solar$data;
  readonly " $fragmentSpreads": FragmentRefs<"SolarChart_solar">;
};

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": null,
  "name": "SolarChart_solar",
  "selections": [
    {
      "alias": null,
      "args": null,
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
        }
      ],
      "storageKey": null
    }
  ],
  "type": "SolarObject",
  "abstractKey": null
};

(node as any).hash = "5de7d6fc04496efcc7e6b23b86620bb7";

export default node;
