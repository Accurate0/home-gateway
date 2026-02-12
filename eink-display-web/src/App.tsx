import SolarChart from "./components/SolarChart";
import ForecastCard from "./components/ForecastCard";
import { graphql, useLazyLoadQuery } from "react-relay";
import type { AppQuery } from "./__generated__/AppQuery.graphql";

const AppQuery = graphql`
  query AppQuery($location: String!, $since: DateTime!) {
    weather(input: { location: $location }) {
      ...ForecastCard_weather
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
      livingRoom {
        temperature
        humidity
      }
      bedroom {
        temperature
        humidity
      }
    }
  }
`;

export default function App() {
  // start of today in RFC3339
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const since = today.toISOString();

  const data = useLazyLoadQuery<AppQuery>(AppQuery, {
    location: "14576",
    since,
  });

  const uvLevel = data?.solar?.current?.uvLevel ?? 0;
  const outdoorTemp = data?.environment?.outdoor?.temperature ?? 20;

  const getUVColor = (uv: number) => {
    if (uv <= 2) return "#16a34a"; // Green (Low)
    if (uv <= 5) return "#facc15"; // Yellow (Moderate)
    if (uv <= 7) return "#fb923c"; // Orange (High)
    if (uv <= 10) return "#dc2626"; // Red (Very High)
    return "#a855f7"; // Purple (Extreme)
  };

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
          <h1 style={{ fontSize: 72, margin: 0, fontWeight: 900, lineHeight: 1, textTransform: "uppercase" }}>
            {new Date().toLocaleDateString([], { weekday: "long" })}
          </h1>
          <div style={{ fontSize: 40, fontWeight: 700, marginTop: 4, color: "black" }}>
            {new Date().toLocaleDateString([], {
              month: "long",
              day: "numeric",
              year: "numeric"
            })}
          </div>
        </div>
        <div
          style={{
            textAlign: "right",
            backgroundColor: "#fefce8", // Fixed light yellow background
            padding: "16px 32px",
            border: "6px solid black",
            display: "flex",
            alignItems: "center",
            gap: 40,
          }}
        >
          <div style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
            <div style={{ fontSize: 80, fontWeight: 900, lineHeight: 1, color: getUVColor(uvLevel) }}>
              {uvLevel.toFixed(1)}
            </div>
            <div style={{ fontSize: 28, fontWeight: 800, color: getUVColor(uvLevel), textTransform: "uppercase", letterSpacing: 1, marginTop: 8 }}>UV INDEX</div>
          </div>
          <div style={{ width: 4, height: 100, backgroundColor: "black" }} />
          <div style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
            <div style={{ fontSize: 80, fontWeight: 900, lineHeight: 1, color: "black" }}>
              {outdoorTemp.toFixed(1)}°
            </div>
            <div style={{ fontSize: 28, fontWeight: 800, color: "black", marginTop: 8, textTransform: "uppercase", letterSpacing: 1 }}>
              {data?.environment?.outdoor?.humidity?.toFixed(0) ?? "--"}% HUM
            </div>
          </div>
        </div>
      </header>

      {/* Main Content Area */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 80, gridColumn: "span 2" }}>
        {/* Left: Solar */}
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
                <div style={{ fontSize: 20, fontWeight: 700, color: "#4b5563" }}>15M AVG</div>
                <div style={{ fontSize: 40, fontWeight: 800 }}>
                  {data?.solar?.current?.statistics?.averages?.last15Mins?.toFixed(0) ?? "--"}W
                </div>
              </div>
              <div style={{ textAlign: "left" }}>
                <div style={{ fontSize: 20, fontWeight: 700, color: "#4b5563" }}>1H AVG</div>
                <div style={{ fontSize: 40, fontWeight: 800 }}>
                  {data?.solar?.current?.statistics?.averages?.last1Hour?.toFixed(0) ?? "--"}W
                </div>
              </div>
            </div>
            <div style={{ textAlign: "right" }}>
              <div style={{ fontSize: 24, fontWeight: 700, color: "#4b5563" }}>TODAY TOTAL</div>
              <div style={{ fontSize: 64, fontWeight: 800 }}>
                {data?.solar?.current?.todayProductionKwh?.toFixed(1) ?? "--"} kWh
              </div>
            </div>
          </div>
          <div style={{ height: 750 }}>
            {data?.solar && (
              <SolarChart solarRef={data.solar} width={850} height={750} />
            )}
          </div>
        </section>

        {/* Right: Forecast */}
        <section style={{ display: "flex", flexDirection: "column" }}>
          {data?.weather && <ForecastCard weatherRef={data.weather} />}
        </section>
      </div>
    </div>
  );
}
