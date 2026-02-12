import { graphql, useFragment } from "react-relay";
import type { WoolworthsCard_woolworths$key } from "./__generated__/WoolworthsCard_woolworths.graphql";

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
  horizontal = false,
}: {
  woolworthsRef: WoolworthsCard_woolworths$key;
  horizontal?: boolean;
}) {
  const data = useFragment(WoolFragment, woolRef);

  const products = data?.products ?? [];

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: horizontal ? "repeat(3, 1fr)" : "1fr",
        gap: horizontal ? 32 : 16,
      }}
    >
      {products.length === 0 && (
        <div style={{ fontSize: 24 }}>No products on special.</div>
      )}
      {products.map((p) => (
        <div
          key={p.name}
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            gap: 16,
            padding: "8px 0",
            borderBottom: "1px solid #e5e7eb",
          }}
        >
          <div style={{ fontSize: 24, fontWeight: 600, flex: 1 }}>{p.name}</div>
          <div style={{ fontSize: 28, fontWeight: 700, color: "#16a34a" }}>
            ${p.price.toFixed(2)}
          </div>
        </div>
      ))}
    </div>
  );
}
