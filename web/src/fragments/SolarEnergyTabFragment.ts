import { graphql } from "react-relay";

export const SOLAR_ENERGY_TAB_FRAGMENT = graphql`
  fragment SolarEnergyTabFragment on QueryRoot {
    solar(input: {since: $since}) {
      current {
        todayProductionKwh
        yesterdayProductionKwh
      }
      history {
        at
        uvLevel
        wh
        timestamp
      }
    }
    energy {
      history(input: {since: $since}) {
        id
        used
        solarExported
        time
      }
    }
  }
`; 