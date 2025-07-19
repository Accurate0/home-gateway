/**
 * @generated SignedSource<<989f7a2be62db6d3300343f2c61c99f4>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
export type ApplianceStateType = "OFF" | "ON" | "%future added value";
export type DoorState = "CLOSED" | "OPEN" | "%future added value";
export type UnifiState = "CONNECTED" | "DISCONNECTED" | "%future added value";
import { FragmentRefs } from "relay-runtime";
export type OverviewTabFragment$data = {
  readonly environment: {
    readonly bedroom: {
      readonly humidity: number;
      readonly pressure: number;
      readonly temperature: number;
    };
    readonly laundry: {
      readonly humidity: number;
      readonly pressure: number;
      readonly temperature: number;
    };
    readonly livingRoom: {
      readonly humidity: number;
      readonly pressure: number;
      readonly temperature: number;
    };
    readonly outdoor: {
      readonly humidity: number;
      readonly pressure: number;
      readonly temperature: number;
    };
  };
  readonly events: {
    readonly appliances: ReadonlyArray<{
      readonly id: any;
      readonly name: string;
      readonly state: ApplianceStateType;
      readonly time: any;
    }>;
    readonly doors: ReadonlyArray<{
      readonly id: any;
      readonly name: string;
      readonly state: DoorState;
      readonly time: any;
    }>;
    readonly wifi: ReadonlyArray<{
      readonly id: any;
      readonly name: string;
      readonly state: UnifiState;
      readonly time: any;
    }>;
  };
  readonly " $fragmentType": "OverviewTabFragment";
};
export type OverviewTabFragment$key = {
  readonly " $data"?: OverviewTabFragment$data;
  readonly " $fragmentSpreads": FragmentRefs<"OverviewTabFragment">;
};

const node: ReaderFragment = (function(){
var v0 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v1 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "time",
  "storageKey": null
},
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "state",
  "storageKey": null
},
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
},
v4 = [
  (v0/*: any*/),
  (v1/*: any*/),
  (v3/*: any*/),
  (v2/*: any*/)
],
v5 = [
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "temperature",
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "humidity",
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "pressure",
    "storageKey": null
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
  "name": "OverviewTabFragment",
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
      "concreteType": "EventsObject",
      "kind": "LinkedField",
      "name": "events",
      "plural": false,
      "selections": [
        {
          "alias": null,
          "args": null,
          "concreteType": "DoorEvent",
          "kind": "LinkedField",
          "name": "doors",
          "plural": true,
          "selections": [
            (v0/*: any*/),
            (v1/*: any*/),
            (v2/*: any*/),
            (v3/*: any*/)
          ],
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "concreteType": "ApplianceEvent",
          "kind": "LinkedField",
          "name": "appliances",
          "plural": true,
          "selections": (v4/*: any*/),
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "concreteType": "WifiEvent",
          "kind": "LinkedField",
          "name": "wifi",
          "plural": true,
          "selections": (v4/*: any*/),
          "storageKey": null
        }
      ],
      "storageKey": null
    },
    {
      "alias": null,
      "args": null,
      "concreteType": "EnvironmentObject",
      "kind": "LinkedField",
      "name": "environment",
      "plural": false,
      "selections": [
        {
          "alias": null,
          "args": null,
          "concreteType": "EnvironmentDetails",
          "kind": "LinkedField",
          "name": "outdoor",
          "plural": false,
          "selections": (v5/*: any*/),
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "concreteType": "EnvironmentDetails",
          "kind": "LinkedField",
          "name": "laundry",
          "plural": false,
          "selections": (v5/*: any*/),
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "concreteType": "EnvironmentDetails",
          "kind": "LinkedField",
          "name": "livingRoom",
          "plural": false,
          "selections": (v5/*: any*/),
          "storageKey": null
        },
        {
          "alias": null,
          "args": null,
          "concreteType": "EnvironmentDetails",
          "kind": "LinkedField",
          "name": "bedroom",
          "plural": false,
          "selections": (v5/*: any*/),
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

(node as any).hash = "aad1bc3603c0e63f74327e5cc7836863";

export default node;
