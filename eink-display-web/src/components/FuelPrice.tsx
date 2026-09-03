import { graphql, useFragment } from "react-relay";
import type { FuelPrice_fuel$key } from "./__generated__/FuelPrice_fuel.graphql";
import { INK, TYPE } from "../theme";

const FuelFragment = graphql`
  fragment FuelPrice_fuel on FuelWatchObject {
    cheapest {
      name
      suburb
      price
    }
  }
`;

export default function FuelPrice({
  fuelRef,
}: {
  fuelRef: FuelPrice_fuel$key | null | undefined;
}) {
  const fuel = useFragment(FuelFragment, fuelRef ?? null);

  const cheapest = fuel?.cheapest;

  return (
    <div style={{ textAlign: "right" }}>
      <div style={{ ...TYPE.label, color: INK }}>ULP 91</div>

      <div style={{ display: "flex", alignItems: "baseline", justifyContent: "flex-end", gap: 12, marginTop: 6 }}>
        <div style={{ fontSize: 64, fontWeight: 900, lineHeight: 1 }}>
          {cheapest ? cheapest.price.toFixed(1) : "—"}
        </div>

        {cheapest && <div style={TYPE.body}>c/L</div>}
      </div>

      {cheapest && (
        <div style={{ ...TYPE.label, marginTop: 6 }}>{cheapest.suburb}</div>
      )}
    </div>
  );
}
