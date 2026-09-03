/**
 * @generated SignedSource<<67246c6b413aad0545bb58bf22955093>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type NextTrainTile_route$data = {
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
  readonly " $fragmentType": "NextTrainTile_route";
};
export type NextTrainTile_route$key = {
  readonly " $data"?: NextTrainTile_route$data;
  readonly " $fragmentSpreads": FragmentRefs<"NextTrainTile_route">;
};

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": null,
  "name": "NextTrainTile_route",
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
      "concreteType": "DepartureObject",
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
  "type": "RouteDeparturesObject",
  "abstractKey": null
};

(node as any).hash = "a4b4d1ea516f41cf338a79fcd1dfe304";

export default node;
