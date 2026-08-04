import SolarChart from "./SolarChart";
import type { SolarChart_solar$key } from "./__generated__/SolarChart_solar.graphql";

type Product = { readonly name: string; readonly price: number };

function ProductChip({ name, price }: { name: string; price: number }) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 4,
        padding: "20px 28px",
        backgroundColor: "white",
        flex: 1,
      }}
    >
      <div
        style={{
          fontSize: 20,
          fontWeight: 900,
          textTransform: "uppercase",
          letterSpacing: 1,
          color: "black",
        }}
      >
        {name}
      </div>
      <div style={{ fontSize: 48, fontWeight: 900, color: "black" }}>
        ${price.toFixed(2)}
      </div>
    </div>
  );
}

export default function SolarSection({
  solarRef,
  last15Mins,
  last1Hour,
  todayProductionKwh,
  products,
  chartWidth,
  chartHeight,
  lastUpdated,
}: {
  solarRef: SolarChart_solar$key;
  last15Mins: number | null | undefined;
  last1Hour: number | null | undefined;
  todayProductionKwh: number | null | undefined;
  products: readonly Product[];
  chartWidth: number;
  chartHeight: number;
  lastUpdated: string;
}) {
  const energyDrinks = products.filter(
    (p) =>
      (p.name.toLowerCase().includes("red bull") ||
        p.name.toLowerCase().includes("mother energy")) &&
      !p.name.toLowerCase().includes("4 pack"),
  );

  return (
    <section style={{ display: "flex", flexDirection: "column" }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "flex-end",
          marginBottom: 24,
        }}
      >
        <div style={{ display: "flex", gap: 32 }}>
          <div style={{ textAlign: "left" }}>
            <div style={{ fontSize: 20, fontWeight: 700, color: "#4b5563" }}>
              15M AVG
            </div>
            <div style={{ fontSize: 40, fontWeight: 800 }}>
              {last15Mins?.toFixed(0) ?? "--"}W
            </div>
          </div>
          <div style={{ textAlign: "left" }}>
            <div style={{ fontSize: 20, fontWeight: 700, color: "#4b5563" }}>
              1H AVG
            </div>
            <div style={{ fontSize: 40, fontWeight: 800 }}>
              {last1Hour?.toFixed(0) ?? "--"}W
            </div>
          </div>
        </div>
        <div style={{ textAlign: "right" }}>
          <div style={{ fontSize: 24, fontWeight: 700, color: "#4b5563" }}>
            TOTAL
          </div>
          <div style={{ fontSize: 64, fontWeight: 800 }}>
            {todayProductionKwh?.toFixed(1) ?? "--"} kWh
          </div>
        </div>
      </div>

      <div style={{ height: chartHeight }}>
        <SolarChart
          solarRef={solarRef}
          width={chartWidth}
          height={chartHeight}
        />
      </div>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 20,
          marginTop: 8,
        }}
      >
        {energyDrinks.map((p) => (
          <ProductChip key={p.name} name={p.name} price={p.price} />
        ))}
      </div>

      <div
        style={{
          marginTop: 24,
          fontSize: 18,
          fontWeight: 800,
          color: "black",
          textTransform: "uppercase",
          letterSpacing: 1,
        }}
      >
        {lastUpdated}
      </div>
    </section>
  );
}
