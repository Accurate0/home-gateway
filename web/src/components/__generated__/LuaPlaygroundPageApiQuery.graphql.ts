/**
 * @generated SignedSource<<f2fc2085f55ec9c57307abd5e2850bb9>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type LuaPlaygroundPageApiQuery$variables = Record<PropertyKey, never>;
export type LuaPlaygroundPageApiQuery$data = {
  readonly luaApi: {
    readonly aliases: ReadonlyArray<{
      readonly name: string;
      readonly type: string;
    }>;
    readonly classes: ReadonlyArray<{
      readonly fields: ReadonlyArray<{
        readonly declaration: string;
        readonly name: string;
        readonly optional: boolean;
        readonly type: string;
      }>;
      readonly name: string;
    }>;
    readonly definitions: string;
    readonly globals: ReadonlyArray<{
      readonly declaration: string;
      readonly name: string;
      readonly optional: boolean;
      readonly type: string;
    }>;
    readonly namespaces: ReadonlyArray<{
      readonly fields: ReadonlyArray<{
        readonly declaration: string;
        readonly name: string;
        readonly optional: boolean;
        readonly type: string;
      }>;
      readonly functions: ReadonlyArray<{
        readonly name: string;
        readonly params: ReadonlyArray<{
          readonly declaration: string;
          readonly name: string;
          readonly optional: boolean;
          readonly type: string;
        }>;
        readonly returns: string | null | undefined;
        readonly scope: string | null | undefined;
        readonly signature: string;
      }>;
      readonly name: string;
    }>;
  };
};
export type LuaPlaygroundPageApiQuery = {
  response: LuaPlaygroundPageApiQuery$data;
  variables: LuaPlaygroundPageApiQuery$variables;
};

const node: ConcreteRequest = (function(){
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
  "name": "type",
  "storageKey": null
},
v2 = [
  (v0/*:: as any*/),
  (v1/*:: as any*/),
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "optional",
    "storageKey": null
  },
  {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "declaration",
    "storageKey": null
  }
],
v3 = {
  "alias": null,
  "args": null,
  "concreteType": "LuaFieldObject",
  "kind": "LinkedField",
  "name": "fields",
  "plural": true,
  "selections": (v2/*:: as any*/),
  "storageKey": null
},
v4 = [
  {
    "alias": null,
    "args": null,
    "concreteType": "LuaApiObject",
    "kind": "LinkedField",
    "name": "luaApi",
    "plural": false,
    "selections": [
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "definitions",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "LuaFieldObject",
        "kind": "LinkedField",
        "name": "globals",
        "plural": true,
        "selections": (v2/*:: as any*/),
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "LuaAliasObject",
        "kind": "LinkedField",
        "name": "aliases",
        "plural": true,
        "selections": [
          (v0/*:: as any*/),
          (v1/*:: as any*/)
        ],
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "LuaClassObject",
        "kind": "LinkedField",
        "name": "classes",
        "plural": true,
        "selections": [
          (v0/*:: as any*/),
          (v3/*:: as any*/)
        ],
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "concreteType": "LuaNamespaceObject",
        "kind": "LinkedField",
        "name": "namespaces",
        "plural": true,
        "selections": [
          (v0/*:: as any*/),
          (v3/*:: as any*/),
          {
            "alias": null,
            "args": null,
            "concreteType": "LuaFunctionObject",
            "kind": "LinkedField",
            "name": "functions",
            "plural": true,
            "selections": [
              (v0/*:: as any*/),
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "returns",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "scope",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "kind": "ScalarField",
                "name": "signature",
                "storageKey": null
              },
              {
                "alias": null,
                "args": null,
                "concreteType": "LuaFieldObject",
                "kind": "LinkedField",
                "name": "params",
                "plural": true,
                "selections": (v2/*:: as any*/),
                "storageKey": null
              }
            ],
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
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "LuaPlaygroundPageApiQuery",
    "selections": (v4/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "LuaPlaygroundPageApiQuery",
    "selections": (v4/*:: as any*/)
  },
  "params": {
    "cacheID": "9235e41dc955ab30b8bc78232055fbd7",
    "id": null,
    "metadata": {},
    "name": "LuaPlaygroundPageApiQuery",
    "operationKind": "query",
    "text": "query LuaPlaygroundPageApiQuery {\n  luaApi {\n    definitions\n    globals {\n      name\n      type\n      optional\n      declaration\n    }\n    aliases {\n      name\n      type\n    }\n    classes {\n      name\n      fields {\n        name\n        type\n        optional\n        declaration\n      }\n    }\n    namespaces {\n      name\n      fields {\n        name\n        type\n        optional\n        declaration\n      }\n      functions {\n        name\n        returns\n        scope\n        signature\n        params {\n          name\n          type\n          optional\n          declaration\n        }\n      }\n    }\n  }\n}\n"
  }
};
})();

(node as any).hash = "cc0dde76300f0e136109febff5cca46f";

export default node;
