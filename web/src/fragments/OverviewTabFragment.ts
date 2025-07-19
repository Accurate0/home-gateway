import { graphql } from "react-relay";

export const OVERVIEW_TAB_FRAGMENT = graphql`
  fragment OverviewTabFragment on QueryRoot {
    events(input: { since: $since }) {
      doors {
        name
        time
        state
        id
      }
      appliances {
        name
        time
        id
        state
      }
      wifi {
        name
        time
        id
        state
      }
    }
    environment {
      outdoor {
        temperature
        humidity
        pressure
      }
      laundry {
        temperature
        humidity
        pressure
      }
      livingRoom {
        temperature
        humidity
        pressure
      }
      bedroom {
        temperature
        humidity
        pressure
      }
    }
  }
`; 