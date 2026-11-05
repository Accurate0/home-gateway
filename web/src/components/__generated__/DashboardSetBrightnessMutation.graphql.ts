/**
 * @generated SignedSource<<c21162d402f7bd32ad854beec37ab8b6>>
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
  readonly homeLight: {
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
    "concreteType": "HomeLightMutation",
    "kind": "LinkedField",
    "name": "homeLight",
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
    "type": "Mutation",
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
    "cacheID": "d273d3d349c49a9a9ed5e70cb4fcafda",
    "id": null,
    "metadata": {},
    "name": "DashboardSetBrightnessMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardSetBrightnessMutation(\n  $id: IdOrAlias!\n  $value: Int!\n) {\n  homeLight(id: $id) {\n    setBrightness(input: {value: $value})\n  }\n}\n"
  }
};
})();

(node as any).hash = "74bbbae6e17a94f904e93cd4a8bc82ac";

export default node;
