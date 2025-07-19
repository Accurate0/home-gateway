import { graphql } from "react-relay";

// Main GraphQL query using fragments
export const APP_QUERY = graphql`
  query AppQuery($since: DateTime!) {
    ...OverviewTabFragment
    ...SolarEnergyTabFragment
  }
`; 