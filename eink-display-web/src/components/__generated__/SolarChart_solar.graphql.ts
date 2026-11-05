/**
 * @generated SignedSource<<e1d9c6a112f9541ebe47449e7355c45d>>
 * @lightSyntaxTransform
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
    readonly uvLevel: number | null | undefined;
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
        {
          "alias": null,
          "args": null,
          "kind": "ScalarField",
          "name": "uvLevel",
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "type": "HomeSolarObject",
  "abstractKey": null
};

(node as any).hash = "7c7e88650b8705d6f5e1056e55fca8e4";

export default node;
