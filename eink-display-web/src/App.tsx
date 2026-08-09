import SolarSection from "./components/SolarSection";
import ForecastCard from "./components/ForecastCard";
import ClimateBand from "./components/ClimateBand";
import Header from "./components/Header";
import PanelBattery from "./components/PanelBattery";
import Frame from "./components/Frame";
import { graphql, useLazyLoadQuery } from "react-relay";
import type { AppQuery } from "./__generated__/AppQuery.graphql";
import { formatUpdatedAt, perthMidnightISO } from "./lib/time";
import { CONTENT_H, ROW } from "./theme";

const AppQuery = graphql`
  query AppQuery($location: String!, $since: DateTime!) {
    weather(input: { location: $location }) {
      ...ClimateBand_weather
      ...ForecastCard_weather
    }
    solar {
      current {
        todayProductionKwh
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
    hallwayPanel: einkDisplay(id: "hallway-epd") {
      name
      batteryPercentage
    }
    livingRoomPanel: einkDisplay(id: "living-room-epd") {
      name
      batteryPercentage
    }
  }
`;

const LOCATION = "14576";

const view = new URLSearchParams(window.location.search).get("view") ?? "home";

const WEATHER_FORECAST_H = CONTENT_H - ROW.header - ROW.climate - ROW.footer;

export default function App() {
  const data = useLazyLoadQuery<AppQuery>(AppQuery, {
    location: LOCATION,
    since: perthMidnightISO(),
  });

  const current = data.solar?.current;
  const weatherOnly = view === "weather";

  return (
    <Frame>
      <Header updatedAt={formatUpdatedAt(new Date())} stale={false} />

      <ClimateBand outdoor={data.outdoor} weatherRef={data.weather} />

      {!weatherOnly && data.solar && (
        <SolarSection
          solarRef={data.solar}
          last15Mins={current?.statistics?.averages?.last15Mins}
          last1Hour={current?.statistics?.averages?.last1Hour}
          uvLevel={current?.uvLevel}
          todayProductionKwh={current?.todayProductionKwh}
        />
      )}

      {data.weather && (
        <ForecastCard
          weatherRef={data.weather}
          height={weatherOnly ? WEATHER_FORECAST_H : ROW.forecast}
        />
      )}

      <PanelBattery hallway={data.hallwayPanel} livingRoom={data.livingRoomPanel} />
    </Frame>
  );
}
