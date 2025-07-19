/**
 * @generated SignedSource<<429e2e094f3b7881cf596767fbf23964>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type SolarEnergyTabFragment$data = {
  readonly energy: {
    readonly history: ReadonlyArray<{
      readonly id: any;
      readonly solarExported: number;
      readonly time: any;
      readonly used: number;
    }>;
  };
  readonly solar: {
    readonly current: {
      readonly todayProductionKwh: number;
      readonly yesterdayProductionKwh: number;
    };
    readonly history: ReadonlyArray<{
      readonly at: any;
      readonly timestamp: number;
      readonly uvLevel: number | null | undefined;
      readonly wh: number;
    }>;
  };
  readonly " $fragmentType": "SolarEnergyTabFragment";
};
export type SolarEnergyTabFragment$key = {
  readonly " $data"?: SolarEnergyTabFragment$data;
  readonly " $fragmentSpreads": FragmentRefs<"SolarEnergyTabFragment">;
};

const node: ReaderFragment = (function(){
var v0 = [
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
];
return {
  "argumentDefinitions": [
    {
      "kind": "RootArgument",
      "name": "since"
    }
  ],
  "kind": "Fragment",
  "metadata": null,
  "name": "SolarEnergyTabFragment",
  "selections": [
    {
      "alias": null,
      "args": (v0/*: any*/),
      "concreteType": "SolarObject",
      "kind": "LinkedField",
      "name": "solar",
      "plural": false,
      "selections": [
        {
          "alias": null,
          "args": null,
          "concreteType": "SolarCurrentResponse",
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
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "yesterdayProductionKwh",
              "storageKey": null
            }
          ],
          "storageKey": null
        },
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
              "name": "at",
              "storageKey": null
            },
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "uvLevel",
              "storageKey": null
            },
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
              "name": "timestamp",
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
      "concreteType": "EnergyObject",
      "kind": "LinkedField",
      "name": "energy",
      "plural": false,
      "selections": [
        {
          "alias": null,
          "args": (v0/*: any*/),
          "concreteType": "EnergyConsumption",
          "kind": "LinkedField",
          "name": "history",
          "plural": true,
          "selections": [
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "id",
              "storageKey": null
            },
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "used",
              "storageKey": null
            },
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "solarExported",
              "storageKey": null
            },
            {
              "alias": null,
              "args": null,
              "kind": "ScalarField",
              "name": "time",
              "storageKey": null
            }
          ],
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "type": "QueryRoot",
  "abstractKey": null
};
})();

(node as any).hash = "1f339e33bb92b43580ec0bc257c7ef47";

export default node;
