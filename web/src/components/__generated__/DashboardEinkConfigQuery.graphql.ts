/**
 * @generated SignedSource<<f66265c2e67d928c83a4b1c138b77932>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardEinkConfigQuery$variables = {
  id: string;
};
export type DashboardEinkConfigQuery$data = {
  readonly einkDisplay: {
    readonly config: {
      readonly grace: string;
      readonly lead: string | null | undefined;
    } | null | undefined;
    readonly deviceConfig: {
      readonly clearScreen: boolean | null | undefined;
      readonly imageUrl: string | null | undefined;
      readonly refreshIntervalMins: number | null | undefined;
    };
    readonly isCharging: boolean | null | undefined;
    readonly lastSeen: any | null | undefined;
    readonly nextWakeAt: any | null | undefined;
    readonly partialRefreshCount: number | null | undefined;
    readonly rtcDriftSecs: number | null | undefined;
    readonly rtcReportedAt: any | null | undefined;
    readonly rtcReportedOffsetSecs: number | null | undefined;
    readonly rtcSyncDueAt: any | null | undefined;
    readonly rtcSyncedAt: any | null | undefined;
    readonly targetFirmwareVersion: string | null | undefined;
  };
};
export type DashboardEinkConfigQuery = {
  response: DashboardEinkConfigQuery$data;
  variables: DashboardEinkConfigQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "id"
  }
],
v1 = [
  {
    "alias": null,
    "args": [
      {
        "kind": "Variable",
        "name": "id",
        "variableName": "id"
      }
    ],
    "concreteType": "EinkDisplayEntity",
    "kind": "LinkedField",
    "name": "einkDisplay",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "EpdConfig",
        "kind": "LinkedField",
        "name": "deviceConfig",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "refreshIntervalMins",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "imageUrl",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "clearScreen",
            "storageKey": null
          }
        ],
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "lastSeen",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "nextWakeAt",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "partialRefreshCount",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "targetFirmwareVersion",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "isCharging",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "rtcReportedAt",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "rtcReportedOffsetSecs",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "rtcSyncedAt",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "rtcDriftSecs",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "rtcSyncDueAt",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "EinkDisplayConfig",
        "kind": "LinkedField",
        "name": "config",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "grace",
            "storageKey": null
          },
          {
            "alias": null,
            "args": null,
            "kind": "ScalarField",
            "name": "lead",
            "storageKey": null
          }
        ],
        "storageKey": null
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
    "name": "DashboardEinkConfigQuery",
    "selections": (v1/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardEinkConfigQuery",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "6aba3c55fa43cc71f7ed2dd1b70d5871",
    "id": null,
    "metadata": {},
    "name": "DashboardEinkConfigQuery",
    "operationKind": "query",
    "text": "query DashboardEinkConfigQuery(\n  $id: IdOrAlias!\n) {\n  einkDisplay(id: $id) {\n    deviceConfig {\n      refreshIntervalMins\n      imageUrl\n      clearScreen\n    }\n    lastSeen\n    nextWakeAt\n    partialRefreshCount\n    targetFirmwareVersion\n    isCharging\n    rtcReportedAt\n    rtcReportedOffsetSecs\n    rtcSyncedAt\n    rtcDriftSecs\n    rtcSyncDueAt\n    config {\n      grace\n      lead\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "3822def2b521eb363deb0017d5f29968";

export default node;
