import { graphql, useFragment } from "react-relay";
import type { WoolworthsCard_woolworths$key } from "./__generated__/WoolworthsCard_woolworths.graphql";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

const WoolFragment = graphql`
  fragment WoolworthsCard_woolworths on WoolworthsObject {
    products {
      name
      price
    }
  }
`;

export default function WoolworthsCard({
  woolworthsRef: woolRef,
}: {
  woolworthsRef: WoolworthsCard_woolworths$key;
}) {
  const data = useFragment(WoolFragment, woolRef);

  const products = data?.products ?? [];

  return (
    <Card style={{ width: 420 }}>
      <CardHeader>
        <div>
          <CardTitle>Woolworths — Products</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {products.length === 0 && <div>No products.</div>}
          {products.map((p) => (
            <div
              key={p.name}
              style={{
                display: "flex",
                justifyContent: "space-between",
                gap: 8,
              }}
            >
              <div style={{ fontWeight: 600 }}>{p.name}</div>
              <div style={{ color: "#374151" }}>
                {typeof p.price === "number" ? p.price.toFixed(2) : p.price}
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
