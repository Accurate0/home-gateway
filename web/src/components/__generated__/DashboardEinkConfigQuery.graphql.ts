/**
 * @generated SignedSource<<a2c88207236dd358e7494b1c6efa3a03>>
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
    readonly deviceConfig: {
      readonly clearScreen: boolean | null | undefined;
      readonly imageUrl: string | null | undefined;
      readonly refreshIntervalMins: number | null | undefined;
    };
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
    "cacheID": "6fd2e19681f623ea9c1daefcbfb880db",
    "id": null,
    "metadata": {},
    "name": "DashboardEinkConfigQuery",
    "operationKind": "query",
    "text": "query DashboardEinkConfigQuery(\n  $id: String!\n) {\n  einkDisplay(id: $id) {\n    deviceConfig {\n      refreshIntervalMins\n      imageUrl\n      clearScreen\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "730f7b7f8ee203c4687c8297f008a8e7";

export default node;
