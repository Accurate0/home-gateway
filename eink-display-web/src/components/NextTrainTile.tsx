import { graphql, useFragment } from "react-relay";
import type { NextTrainTile_route$key } from "./__generated__/NextTrainTile_route.graphql";
import FuelPrice from "./FuelPrice";
import type { FuelPrice_fuel$key } from "./__generated__/FuelPrice_fuel.graphql";
import { HAIRLINE, INK, RED, ROW, TYPE } from "../theme";

const RouteFragment = graphql`
  fragment NextTrainTile_route on RouteDeparturesObject {
    origin
    destination
    departures {
      line
      platform
      scheduledDeparture
      delayMinutes
      minutesAway
      live
    }
  }
`;

const LATE_MINUTES = 2;

function clockTime(value: string) {
  return new Date(value).toLocaleTimeString("en-AU", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
    timeZone: "Australia/Perth",
  });
}

export default function NextTrainTile({
  routeRef,
  fuelRef,
}: {
  routeRef: NextTrainTile_route$key | null | undefined;
  fuelRef: FuelPrice_fuel$key | null | undefined;
}) {
  const route = useFragment(RouteFragment, routeRef ?? null);

  const [next, ...rest] = route?.departures ?? [];
  const late = (next?.delayMinutes ?? 0) > LATE_MINUTES;

  return (
    <section
      style={{
        height: ROW.transit,
        borderBottom: HAIRLINE,
        boxSizing: "border-box",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: 40,
      }}
    >
      <div>
        <div style={{ ...TYPE.label, color: INK }}>
          {route ? `${route.origin} \u2192 ${route.destination}` : "Next train"}
        </div>

        <div style={{ display: "flex", alignItems: "baseline", gap: 16, marginTop: 6 }}>
          <div
            style={{
              fontSize: 64,
              fontWeight: 900,
              lineHeight: 1,
              color: late ? RED : INK,
            }}
          >
            {next ? Math.max(next.minutesAway, 0) : "\u2014"}
            {next && <span style={{ fontSize: 30, fontWeight: 800 }}> min</span>}
          </div>

          {next && (
            <div style={TYPE.body}>
              {clockTime(next.scheduledDeparture)}
              {next.platform && ` \u00b7 Plat ${next.platform}`}
              {!next.live && " \u00b7 sched"}
            </div>
          )}
        </div>

        {rest.length > 0 && (
          <div style={{ ...TYPE.label, marginTop: 6 }}>
            then {rest.map((departure) => clockTime(departure.scheduledDeparture)).join(" \u00b7 ")}
          </div>
        )}
      </div>

      <FuelPrice fuelRef={fuelRef} />
    </section>
  );
}
