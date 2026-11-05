/**
 * @generated SignedSource<<14e048692ff6479773c9bbcb77f1ec11>>
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
  readonly homeEinkDisplay: {
    readonly config: {
      readonly grace: string;
      readonly lead: string | null | undefined;
    } | null | undefined;
    readonly deviceConfig: {
      readonly imageUrl: string | null | undefined;
      readonly refreshIntervalMins: number | null | undefined;
    };
    readonly isCharging: boolean | null | undefined;
    readonly lastSeen: any | null | undefined;
    readonly lastWakeTraceUrl: string | null | undefined;
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
    "concreteType": "HomeEinkDisplayEntity",
    "kind": "LinkedField",
    "name": "homeEinkDisplay",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "HomeEpdConfig",
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
        "kind": "ScalarField",
        "name": "lastWakeTraceUrl",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "HomeEinkDisplayConfig",
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
    "type": "Query",
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
    "cacheID": "89d916933e6a78d111c77168973f2025",
    "id": null,
    "metadata": {},
    "name": "DashboardEinkConfigQuery",
    "operationKind": "query",
    "text": "query DashboardEinkConfigQuery(\n  $id: IdOrAlias!\n) {\n  homeEinkDisplay(id: $id) {\n    deviceConfig {\n      refreshIntervalMins\n      imageUrl\n    }\n    lastSeen\n    nextWakeAt\n    partialRefreshCount\n    targetFirmwareVersion\n    isCharging\n    rtcReportedAt\n    rtcReportedOffsetSecs\n    rtcSyncedAt\n    rtcDriftSecs\n    rtcSyncDueAt\n    lastWakeTraceUrl\n    config {\n      grace\n      lead\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "d040e96c3f6581c91faf96117d303e06";

export default node;
