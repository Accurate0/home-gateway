import SolarSection from "./components/SolarSection";
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
    outdoor: environment(id: "env-outdoor") {
      temperature
      humidity
    }
  }
`;

const orientation =
  new URLSearchParams(window.location.search).get("orientation") === "portrait"
    ? "portrait"
    : "landscape";

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

function getUVColor(uv: number) {
  if (uv <= 2) return "#16a34a"; // Green
  if (uv <= 5) return "#f97316"; // Orange
  if (uv <= 7) return "#dc2626"; // Red
  return "#991b1b"; // Deep Red
}

function Header({
  uvLevel,
  outdoorTemp,
  humidity,
}: {
  uvLevel: number;
  outdoorTemp: number;
  humidity: number | null | undefined;
}) {
  return (
    <header
      style={{
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
          }}
        >
          <div
            style={{
              fontSize: 80,
              fontWeight: 900,
              lineHeight: 1,
              color: getUVColor(uvLevel),
            }}
          >
            {uvLevel.toFixed(1)}
          </div>
          <div
            style={{
              fontSize: 28,
              fontWeight: 800,
              color: getUVColor(uvLevel),
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
            {humidity?.toFixed(0) ?? "--"}% HUM
          </div>
        </div>
      </div>
    </header>
  );
}

export default function App() {
  const data = useLazyLoadQuery<AppQuery>(AppQuery, {
    location: "14576",
    since: getLocalMidnightISO(),
  });

  const portrait = orientation === "portrait";

  const uvLevel = data?.solar?.current?.uvLevel ?? 0;
  const outdoorTemp = data?.outdoor?.temperature ?? 20;

  const lastUpdated = new Intl.DateTimeFormat("en-AU", {
    timeZone: "Australia/Perth",
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: true,
  }).format(new Date());

  const solar = data?.solar && (
    <SolarSection
      solarRef={data.solar}
      last15Mins={data.solar.current?.statistics?.averages?.last15Mins}
      last1Hour={data.solar.current?.statistics?.averages?.last1Hour}
      todayProductionKwh={data.solar.current?.todayProductionKwh}
      products={data.woolworths.products}
      chartWidth={portrait ? 1060 : 820}
      chartHeight={portrait ? 440 : 590}
      lastUpdated={lastUpdated}
    />
  );

  const forecast = data?.weather && (
    <ForecastCard weatherRef={data.weather} dense={portrait} />
  );

  return (
    <div
      style={{
        width: "100%",
        minHeight: "100vh",
        backgroundColor: "white",
        color: "black",
        fontFamily: "Inter, system-ui, sans-serif",
        padding: portrait ? "32px 40px" : "40px 60px",
        display: "flex",
        flexDirection: "column",
        gap: portrait ? 20 : 32,
        boxSizing: "border-box",
      }}
    >
      <Header
        uvLevel={uvLevel}
        outdoorTemp={outdoorTemp}
        humidity={data?.outdoor?.humidity}
      />

      {portrait ? (
        <>
          <section>{forecast}</section>
          {solar}
        </>
      ) : (
        <div style={{ display: "flex", gap: 80 }}>
          <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
            {solar}
          </div>
          <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
            <section>{forecast}</section>
          </div>
        </div>
      )}
    </div>
  );
}
