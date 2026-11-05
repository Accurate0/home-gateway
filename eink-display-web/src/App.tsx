import SolarSection from "./components/SolarSection";
import ForecastCard from "./components/ForecastCard";
import ClimateBand from "./components/ClimateBand";
import Header from "./components/Header";
import PanelBattery from "./components/PanelBattery";
import TransitFuelRow from "./components/TransitFuelRow";
import Frame from "./components/Frame";
import { graphql, useLazyLoadQuery } from "react-relay";
import type { AppQuery } from "./__generated__/AppQuery.graphql";
import { formatUpdatedAt, perthMidnightISO } from "./lib/time";
import { CONTENT_H, ROW } from "./theme";

const AppQuery = graphql`
  query AppQuery($location: String!, $since: DateTime!, $transitRoute: String!) {
    homeWeather(input: { location: $location }) {
      ...ClimateBand_weather
      ...ForecastCard_weather
    }
    homeSolar {
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
    homeTransperth {
      route(id: $transitRoute) {
        ...TransitFuelRow_route
      }
    }
    homeFuelwatch {
      ...FuelPrice_fuel
    }
    outdoor: homeEnvironment(id: "env-outdoor") {
      temperature
      humidity
    }
    hallwayPanel: homeEinkDisplay(id: "hallway-epd") {
      name
      batteryPercentage
    }
    livingRoomPanel: homeEinkDisplay(id: "living-room-epd") {
      name
      batteryPercentage
    }
  }
`;

const LOCATION = "14576";

const TRANSIT_ROUTE = "aubin-grove-to-perth";

const view = new URLSearchParams(window.location.search).get("view") ?? "home";

const WEATHER_FORECAST_H = CONTENT_H - ROW.header - ROW.climate - ROW.footer;

export default function App() {
  const data = useLazyLoadQuery<AppQuery>(AppQuery, {
    location: LOCATION,
    since: perthMidnightISO(),
    transitRoute: TRANSIT_ROUTE,
  });

  const current = data.homeSolar?.current;
  const weatherOnly = view === "weather";

  return (
    <Frame>
      <Header updatedAt={formatUpdatedAt(new Date())} stale={false} />

      <ClimateBand outdoor={data.outdoor} weatherRef={data.homeWeather} />

      {!weatherOnly && (
        <TransitFuelRow routeRef={data.homeTransperth?.route} fuelRef={data.homeFuelwatch} />
      )}

      {!weatherOnly && data.homeSolar && (
        <SolarSection
          solarRef={data.homeSolar}
          last15Mins={current?.statistics?.averages?.last15Mins}
          last1Hour={current?.statistics?.averages?.last1Hour}
          uvLevel={current?.uvLevel}
          todayProductionKwh={current?.todayProductionKwh}
        />
      )}

      {data.homeWeather && (
        <ForecastCard
          weatherRef={data.homeWeather}
          height={weatherOnly ? WEATHER_FORECAST_H : ROW.forecast}
        />
      )}

      <PanelBattery hallway={data.hallwayPanel} livingRoom={data.livingRoomPanel} />
    </Frame>
  );
}
