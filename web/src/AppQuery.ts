import { graphql } from "react-relay";

// Updated GraphQL query with solar data
export const APP_QUERY = graphql`
  query AppQuery($since: DateTime!) {
    solar {
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