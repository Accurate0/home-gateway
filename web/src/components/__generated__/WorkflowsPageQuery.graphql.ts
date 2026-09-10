/**
 * @generated SignedSource<<0649e9aa328ffdabf55f2484504fcd98>>
 * @lightSyntaxTransform
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest } from 'relay-runtime';
export type Mode = "AWAY" | "GUEST" | "HOME" | "VACATION" | "%future added value";
export type WorkflowsPageQuery$variables = Record<PropertyKey, never>;
export type WorkflowsPageQuery$data = {
  readonly workflows: ReadonlyArray<{
    readonly configEnabled: boolean;
    readonly dryRun: boolean;
    readonly enabled: boolean;
    readonly group: string;
    readonly id: string;
    readonly modes: ReadonlyArray<Mode>;
    readonly name: string;
    readonly reusable: boolean;
    readonly slug: string;
  }>;
};
export type WorkflowsPageQuery = {
  response: WorkflowsPageQuery$data;
  variables: WorkflowsPageQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "alias": null,
    "args": null,
    "concreteType": "WorkflowStatus",
    "kind": "LinkedField",
    "name": "workflows",
    "plural": true,
    "selections": [
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
        "name": "slug",
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
        "name": "group",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "enabled",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "configEnabled",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "dryRun",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "reusable",
        "storageKey": null
      },
      {
        "alias": null,
        "args": null,
        "kind": "ScalarField",
        "name": "modes",
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
    "name": "WorkflowsPageQuery",
    "selections": (v0/*:: as any*/),
    "type": "QueryRoot",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "WorkflowsPageQuery",
    "selections": (v0/*:: as any*/)
  },
  "params": {
    "cacheID": "d09031b9363f1db4280df45d94197d44",
    "id": null,
    "metadata": {},
    "name": "WorkflowsPageQuery",
    "operationKind": "query",
    "text": "query WorkflowsPageQuery {\n  workflows {\n    id\n    slug\n    name\n    group\n    enabled\n    configEnabled\n    dryRun\n    reusable\n    modes\n  }\n}\n"
  }
};
})();

(node as any).hash = "5dd48f5bb7e8ec9b54603070983fe8d0";

export default node;
