/**
 * @generated SignedSource<<7b6d1f3a49a9028f32179653f66b5f90>>
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
  readonly entities: ReadonlyArray<{
    readonly __typename: "DoorEntity";
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
    readonly __typename: "EinkDisplayEntity";
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
    readonly __typename: "EnvironmentEntity";
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
    readonly __typename: "LightEntity";
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
    readonly __typename: "PresenceEntity";
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
    readonly __typename: "RobotVacuumEntity";
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
    "concreteType": "DeviceBattery",
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
        "concreteType": "BatteryPoint",
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
    "name": "entities",
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
        "type": "LightEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "DoorEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "PresenceEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "EnvironmentEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "EinkDisplayEntity",
        "abstractKey": null
      },
      {
        "kind": "InlineFragment",
        "selections": (v1/*:: as any*/),
        "type": "RobotVacuumEntity",
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
    "type": "QueryRoot",
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
    "cacheID": "9f13313d089ff5f4c4eeb96541e02d35",
    "id": null,
    "metadata": {},
    "name": "BatteryPageQuery",
    "operationKind": "query",
    "text": "query BatteryPageQuery(\n  $since: DateTime!\n) {\n  entities {\n    __typename\n    ... on LightEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on DoorEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on PresenceEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on EnvironmentEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on EinkDisplayEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n    ... on RobotVacuumEntity {\n      id\n      name\n      room\n      battery {\n        history(since: $since) {\n          time\n          batteryPercentage\n        }\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "74c3d172b75837723dcccb2734197365";

export default node;
