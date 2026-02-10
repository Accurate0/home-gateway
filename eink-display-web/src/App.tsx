import SolarChart from "./components/SolarChart";
import ForecastCard from "./components/ForecastCard";
import WoolworthsCard from "./components/WoolworthsCard";
import { graphql, useLazyLoadQuery } from "react-relay";
import type { AppQuery } from "./__generated__/AppQuery.graphql";

const AppQuery = graphql`
  query AppQuery($location: String!, $since: DateTime!) {
    weather(input: { location: $location }) {
      ...ForecastCard_weather
    }
    solar {
      ...SolarChart_solar @arguments(since: $since)
    }
    woolworths {
      ...WoolworthsCard_woolworths
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

  return (
    <>
      <div
        style={{
          display: "flex",
          gap: 24,
          alignItems: "flex-start",
          justifyContent: "center",
          paddingTop: 16,
        }}
      >
        <div>{data?.solar && <SolarChart solarRef={data.solar} />}</div>

        <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
          {data?.weather && <ForecastCard weatherRef={data.weather} />}
          {data?.woolworths && (
            <div style={{ alignSelf: "flex-start" }}>
              <WoolworthsCard woolworthsRef={data.woolworths} />
            </div>
          )}
        </div>
      </div>
    </>
  );
}
