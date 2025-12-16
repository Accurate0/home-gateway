/**
 * @generated SignedSource<<19ca97dfdc97c155c49f9d6e33d7ce4a>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ReaderFragment } from 'relay-runtime';
import { FragmentRefs } from "relay-runtime";
export type WoolworthsCard_woolworths$data = {
  readonly products: ReadonlyArray<{
    readonly name: string;
    readonly price: number;
  }>;
  readonly " $fragmentType": "WoolworthsCard_woolworths";
};
export type WoolworthsCard_woolworths$key = {
  readonly " $data"?: WoolworthsCard_woolworths$data;
  readonly " $fragmentSpreads": FragmentRefs<"WoolworthsCard_woolworths">;
};

const node: ReaderFragment = {
  "argumentDefinitions": [],
  "kind": "Fragment",
  "metadata": null,
  "name": "WoolworthsCard_woolworths",
  "selections": [
    {
      "alias": null,
      "args": null,
      "concreteType": "WoolworthsProducts",
      "kind": "LinkedField",
      "name": "products",
      "plural": true,
      "selections": [
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
          "name": "price",
          "storageKey": null
        }
      ],
      "storageKey": null
    }
  ],
  "type": "WoolworthsObject",
  "abstractKey": null
};

(node as any).hash = "730ba0734eb81bb2718b11d00d4842f5";

export default node;
