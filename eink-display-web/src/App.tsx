import SolarChart from "./components/SolarChart";
import ForecastCard from "./components/ForecastCard";
import { graphql, useLazyLoadQuery } from "react-relay";
import type { AppQuery } from "./__generated__/AppQuery.graphql";

const AppQuery = graphql`
  query AppQuery($location: String!, $since: DateTime!) {
    weather(input: { location: $location }) {
      ...ForecastCard_weather
    }
    woolworths {
      products {
        name
        price
      }
    }
    solar {
      current {
        todayProductionKwh
        currentProductionWh
        uvLevel
        statistics {
          averages {
            last15Mins
            last1Hour
          }
        }
      }
      ...SolarChart_solar @arguments(since: $since)
    }
    environment {
      outdoor {
        temperature
        humidity
      }
    }
  }
`;

function getLocalMidnightISO() {
  const now = new Date();
  const parts = new Intl.DateTimeFormat("en-AU", {
    timeZone: "Australia/Perth",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(now);

  const getPart = (type: string) => parts.find((p) => p.type === type)?.value;
  const year = getPart("year");
  const month = getPart("month");
  const day = getPart("day");

  const midnightPerth = `${year}-${month}-${day}T00:00:00+08:00`;
  const d = new Date(midnightPerth);
  return d.toISOString();
}

function ProductChip({ name, price }: { name: string; price: number }) {
  const isRedBull = name.toLowerCase().includes("red bull");
  const isMother = name.toLowerCase().includes("mother");

  const bgColor = isRedBull ? "#dbeafe" : isMother ? "#fee2e2" : "white";

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 4,
        padding: "20px 28px",
        backgroundColor: bgColor,
        flex: 1,
        borderRadius: 8,
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
      <div style={{ fontSize: 48, fontWeight: 900, color: "#16a34a" }}>
        ${price.toFixed(2)}
      </div>
    </div>
  );
}

export default function App() {
  const data = useLazyLoadQuery<AppQuery>(AppQuery, {
    location: "14576",
    since: getLocalMidnightISO(),
  });

  const uvLevel = data?.solar?.current?.uvLevel ?? 0;
  const outdoorTemp = data?.environment?.outdoor?.temperature ?? 20;

  const getUVBackgroundColor = (uv: number) => {
    if (uv <= 2) return "#dcfce7"; // Light Green
    if (uv <= 5) return "#ffedd5"; // Light Orange
    if (uv <= 7) return "#fee2e2"; // Light Red
    return "#fecaca"; // Light Deep Red
  };

  const lastUpdated = new Intl.DateTimeFormat("en-AU", {
    timeZone: "Australia/Perth",
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: true,
  }).format(new Date());

  return (
    <div
      style={{
        minHeight: "100vh",
        backgroundColor: "white",
        color: "black",
        fontFamily: "Inter, system-ui, sans-serif",
        padding: "40px 60px",
        display: "grid",
        gridTemplateColumns: "1fr 1fr",
        gridTemplateRows: "auto 1fr",
        gap: "32px 60px", // Reduced vertical gap to 32px, kept horizontal at 60px
        boxSizing: "border-box",
        maxWidth: 1600,
        margin: "0 auto",
        position: "relative",
      }}
    >
      {/* Header */}
      <header
        style={{
          gridColumn: "span 2",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          borderBottom: "4px solid black",
          paddingBottom: 16,
        }}
      >
        <div>
          <h1
            style={{
              fontSize: 72,
              margin: 0,
              fontWeight: 900,
              lineHeight: 1,
              textTransform: "uppercase",
            }}
          >
            {new Date().toLocaleDateString([], { weekday: "long" })}
          </h1>
          <div
            style={{
              fontSize: 40,
              fontWeight: 700,
              marginTop: 4,
              color: "black",
            }}
          >
            {new Date().toLocaleDateString([], {
              month: "long",
              day: "numeric",
              year: "numeric",
            })}
          </div>
        </div>
        <div
          style={{
            justifySelf: "end",
            display: "flex",
            alignItems: "center",
            gap: 24,
          }}
        >
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              backgroundColor: getUVBackgroundColor(uvLevel),
              padding: "12px 24px",
              borderRadius: 12,
            }}
          >
            <div
              style={{
                fontSize: 80,
                fontWeight: 900,
                lineHeight: 1,
                color: "black",
              }}
            >
              {uvLevel.toFixed(1)}
            </div>
            <div
              style={{
                fontSize: 28,
                fontWeight: 800,
                color: "black",
                textTransform: "uppercase",
                letterSpacing: 1,
                marginTop: 8,
              }}
            >
              UV INDEX
            </div>
          </div>
          <div style={{ width: 4, height: 100, backgroundColor: "black" }} />
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
            }}
          >
            <div
              style={{
                fontSize: 80,
                fontWeight: 900,
                lineHeight: 1,
                color: "black",
              }}
            >
              {outdoorTemp.toFixed(1)}°
            </div>
            <div
              style={{
                fontSize: 28,
                fontWeight: 800,
                color: "black",
                marginTop: 8,
                textTransform: "uppercase",
                letterSpacing: 1,
              }}
            >
              {data?.environment?.outdoor?.humidity?.toFixed(0) ?? "--"}% HUM
            </div>
          </div>
        </div>
      </header>


      {/* Main Content Area */}
      <div style={{ display: "flex", gap: 80, gridColumn: "span 2" }}>
        {/* Left Column: Solar & Energy Drinks */}
        <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
          <section>
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
                  <div
                    style={{ fontSize: 20, fontWeight: 700, color: "#4b5563" }}
                  >
                    15M AVG
                  </div>
                  <div style={{ fontSize: 40, fontWeight: 800 }}>
                    {data?.solar?.current?.statistics?.averages?.last15Mins?.toFixed(
                      0,
                    ) ?? "--"}
                    W
                  </div>
                </div>
                <div style={{ textAlign: "left" }}>
                  <div
                    style={{ fontSize: 20, fontWeight: 700, color: "#4b5563" }}
                  >
                    1H AVG
                  </div>
                  <div style={{ fontSize: 40, fontWeight: 800 }}>
                    {data?.solar?.current?.statistics?.averages?.last1Hour?.toFixed(
                      0,
                    ) ?? "--"}
                    W
                  </div>
                </div>
              </div>
              <div style={{ textAlign: "right" }}>
                <div
                  style={{ fontSize: 24, fontWeight: 700, color: "#4b5563" }}
                >
                  TODAY TOTAL
                </div>
                <div style={{ fontSize: 64, fontWeight: 800 }}>
                  {data?.solar?.current?.todayProductionKwh?.toFixed(1) ?? "--"}{" "}
                  kWh
                </div>
              </div>
            </div>
            <div style={{ height: 480 }}>
              {data?.solar && (
                <SolarChart solarRef={data.solar} width={820} height={750} />
              )}
            </div>

            <div
              style={{
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: 20,
                marginTop: 8,
              }}
            >
              {data.woolworths.products
                .filter(
                  (p) =>
                    (p.name.toLowerCase().includes("red bull") ||
                      p.name.toLowerCase().includes("mother energy")) &&
                    !p.name.toLowerCase().includes("4 pack"),
                )
                .map((p) => (
                  <ProductChip key={p.name} name={p.name} price={p.price} />
                ))}
            </div>

            {/* Last Updated */}
            <div
              style={{
                marginTop: 32,
                fontSize: 24,
                fontWeight: 800,
                color: "black",
                textTransform: "uppercase",
                letterSpacing: 1,
              }}
            >
              {lastUpdated}
            </div>
          </section>
        </div>

        {/* Right Column: Forecast */}
        <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
          <section>
            {data?.weather && <ForecastCard weatherRef={data.weather} />}
          </section>
        </div>
      </div>
    </div>

  );
}
