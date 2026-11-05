/**
 * @generated SignedSource<<7fcc2fd4bfee779f69f300a1b32b3c2f>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type BatteryPageQuery$variables = {
  since: any;
};
export type BatteryPageQuery$data = {
  readonly homeEntities: ReadonlyArray<{
    readonly __typename: "HomeDoorEntity";
    readonly battery: {
      readonly history: ReadonlyArray<{
        readonly batteryPercentage: number | null | undefined;
        readonly time: any;
      }>;
    } | null | undefined;
    readonly id: string;
    readonly name: string;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "HomeEinkDisplayEntity";
    readonly battery: {
      readonly history: ReadonlyArray<{
        readonly batteryPercentage: number | null | undefined;
        readonly time: any;
      }>;
    } | null | undefined;
    readonly id: string;
    readonly name: string;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "HomeEnvironmentEntity";
    readonly battery: {
      readonly history: ReadonlyArray<{
        readonly batteryPercentage: number | null | undefined;
        readonly time: any;
      }>;
    } | null | undefined;
    readonly id: string;
    readonly name: string;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "HomeGarageDoorEntity";
    readonly battery: {
      readonly history: ReadonlyArray<{
        readonly batteryPercentage: number | null | undefined;
        readonly time: any;
      }>;
    } | null | undefined;
    readonly id: string;
    readonly name: string;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "HomeLightEntity";
    readonly battery: {
      readonly history: ReadonlyArray<{
        readonly batteryPercentage: number | null | undefined;
        readonly time: any;
      }>;
    } | null | undefined;
    readonly id: string;
    readonly name: string;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "HomePresenceEntity";
    readonly battery: {
      readonly history: ReadonlyArray<{
        readonly batteryPercentage: number | null | undefined;
        readonly time: any;
      }>;
    } | null | undefined;
    readonly id: string;
    readonly name: string;
    readonly room: string | null | undefined;
  } | {
    readonly __typename: "HomeRobotVacuumEntity";
    readonly battery: {
      readonly history: ReadonlyArray<{
        readonly batteryPercentage: number | null | undefined;
        readonly time: any;
      }>;
    } | null | undefined;
    readonly id: string;
    readonly name: string;
    readonly room: string | null | undefined;
  } | {
    // This will never be '%other', but we need some
    // value in case none of the concrete values match.
    readonly __typename: "%other";
  }>;
};
export type BatteryPageQuery = {
  response: BatteryPageQuery$data;
  variables: BatteryPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "since"
  }
],
v1 = [
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
    "name": "name",
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "room",
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "concreteType": "HomeDeviceBattery",
    "kind": "LinkedField",
    "name": "battery",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": [
          {
            "kind": "Variable",
            "name": "since",
            "variableName": "since"
          }
        ],
        "concreteType": "HomeBatteryPoint",
        "kind": "LinkedField",
        "name": "history",
        "plural": true,
        "selections": [
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "time",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "batteryPercentage",
            "storageKey": null
          }
        ],
        "storageKey": null
      }
    ],
    "storageKey": null
  }
],
v2 = [
  {
    "alias": null,
    "args": null,
    "concreteType": null,
    "kind": "LinkedField",
    "name": "homeEntities",
    "plural": true,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "__typename",
        "storageKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "HomeLightEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "HomeDoorEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "HomePresenceEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "HomeEnvironmentEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "HomeEinkDisplayEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "HomeRobotVacuumEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "HomeGarageDoorEntity",
        "abstractKey": null
      }
    ],
    "storageKey": null
  }
];
return {
  "fragment": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "BatteryPageQuery",
    "selections": (v2/*:: as any*/),
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "BatteryPageQuery",
    "selections": (v2/*:: as any*/)
  },
  "params": {
    "cacheID": "c18c8a1ea4f4424daf04d2062fac082a",
    "id": null,
    "metadata": {},
    "name": "BatteryPageQuery",
    "operationKind": "query",
    "text": "query BatteryPageQuery(\n  $since: DateTime!\n) {\n  homeEntities {\n    __typename\n    ... on HomeLightEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on HomeDoorEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on HomePresenceEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on HomeEnvironmentEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on HomeEinkDisplayEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on HomeRobotVacuumEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on HomeGarageDoorEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "96bb3738f9c1fe89b27cc7f91939a8cd";

export default node;
