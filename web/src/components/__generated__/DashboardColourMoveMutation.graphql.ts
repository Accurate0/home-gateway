/**
 * @generated SignedSource<<5af48acbf988d57ece98b4f5eb2859bb>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type DashboardColourMoveMutation$variables = {
  id: string;
  value: number;
};
export type DashboardColourMoveMutation$data = {
  readonly light: {
    readonly colourTemperatureMove: boolean;
  };
};
export type DashboardColourMoveMutation = {
  response: DashboardColourMoveMutation$data;
  variables: DashboardColourMoveMutation$variables;
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
        "name": "colourTemperatureMove",
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
    "name": "DashboardColourMoveMutation",
    "selections": (v1/*:: as any*/),
    "type": "MutationRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": (v0/*:: as any*/),
    "kind": "Operation",
    "name": "DashboardColourMoveMutation",
    "selections": (v1/*:: as any*/)
  },
  "params": {
    "cacheID": "070b9582822adb88a022d070f9482fd9",
    "id": null,
    "metadata": {},
    "name": "DashboardColourMoveMutation",
    "operationKind": "mutation",
    "text": "mutation DashboardColourMoveMutation(\n  $id: IdOrAlias!\n  $value: Int!\n) {\n  light(id: $id) {\n    colourTemperatureMove(input: {value: $value})\n  }\n}\n"
  }
};
})();

(node as any).hash = "ce410f1aa3ca4130801c630dc4c97640";

export default node;
