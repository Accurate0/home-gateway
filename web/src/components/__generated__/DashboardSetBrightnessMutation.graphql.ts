/**
 * @generated SignedSource<<a14a57a85b097b88d2f67bda49286959>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardSetBrightnessMutation$variables = {
  id: string;
  value: number;
};
export type DashboardSetBrightnessMutation$data = {
  readonly light: {
    readonly setBrightness: boolean;
  };
};
export type DashboardSetBrightnessMutation = {
  response: DashboardSetBrightnessMutation$data;
  variables: DashboardSetBrightnessMutation$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "id"
  },
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "value"
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
    "concreteType": "LightMutation",
    "kind": "LinkedField",
    "name": "light",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": [
          {
            "fields": [
              {
                "kind": "Variable",
                "name": "value",
                "variableName": "value"
              }
            ],
            "kind": "ObjectValue",
            "name": "input"
          }
        ],
        "kind": "ScalarField",
        "name": "setBrightness",
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
    "name": "DashboardSetBrightnessMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardSetBrightnessMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "ff0a705a85c34194f4439578ba783c13",
    "id": null,
    "metadata": {},
    "name": "DashboardSetBrightnessMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetBrightnessMutation(\n  $id: IdOrAlias!\n  $value: Int!\n) {\n  light(id: $id) {\n    setBrightness(input: {value: $value})\n  }\n}\n"
  }
};
})();

(node as any).hash = "4e384bcfe78475ef821bff90252e5549";

export default node;
