import { graphql, useFragment } from "react-relay";
import type { NextTrainTile_route$key } from "./__generated__/NextTrainTile_route.graphql";
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
}: {
  routeRef: NextTrainTile_route$key | null | undefined;
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
        gap: 24,
      }}
    >
      <div>
        <div style={{ ...TYPE.label, color: INK }}>
          {route ? `${route.origin} → ${route.destination}` : "Next train"}
        </div>

        <div style={{ display: "flex", alignItems: "baseline", gap: 16, marginTop: 8 }}>
          <div
            style={{
              fontSize: 84,
              fontWeight: 900,
              lineHeight: 1,
              color: late ? RED : INK,
            }}
          >
            {next ? Math.max(next.minutesAway, 0) : "—"}
            {next && <span style={{ fontSize: 34, fontWeight: 800 }}> min</span>}
          </div>

          {next && (
            <div style={TYPE.body}>
              {clockTime(next.scheduledDeparture)}
              {next.platform && ` · Plat ${next.platform}`}
              {!next.live && " · sched"}
            </div>
          )}
        </div>
      </div>

      <div style={{ textAlign: "right" }}>
        {rest.map((departure) => (
          <div key={departure.scheduledDeparture} style={{ ...TYPE.body, marginTop: 6 }}>
            {clockTime(departure.scheduledDeparture)}
            <span style={{ ...TYPE.label, marginLeft: 12 }}>
              {Math.max(departure.minutesAway, 0)} min
            </span>
          </div>
        ))}
      </div>
    </section>
  );
}
