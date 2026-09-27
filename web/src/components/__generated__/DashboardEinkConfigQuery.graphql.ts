/**
 * @generated SignedSource<<d767049b8964954aa0c78ba9fbfe2f63>>
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
    "cacheID": "4b249140d312818864755b25180eca54",
    "id": null,
    "metadata": {},
    "name": "DashboardEinkConfigQuery",
    "operationKind": "query",
    "text": "query DashboardEinkConfigQuery(\n  $id: IdOrAlias!\n) {\n  einkDisplay(id: $id) {\n    deviceConfig {\n      refreshIntervalMins\n      imageUrl\n      clearScreen\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "52221abde1cca51858ab8c131d2d7fdc";

export default node;
