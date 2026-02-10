/**
 * @generated SignedSource<<91a26f0e0b930f689b3e03700682ffd5>>
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
  "argumentDefinitions": [
    {
      "defaultValue": null,
      "kind": "LocalArgument",
      "name": "since"
    }
  ],
  "kind": "Fragment",
  "metadata": null,
  "name": "SolarChart_solar",
  "selections": [
    {
      "alias": null,
      "args": [
        {
          "fields": [
            {
              "kind": "Variable",
              "name": "since",
              "variableName": "since"
            }
          ],
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
        }
      ],
      "storageKey": null
    }
  ],
  "type": "SolarObject",
  "abstractKey": null
};

(node as any).hash = "5e69cf357892362271ae752fc29556fd";

export default node;
