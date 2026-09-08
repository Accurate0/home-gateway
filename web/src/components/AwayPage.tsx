import { useMemo, useState } from "react";
import { graphql, useLazyLoadQuery, useMutation } from "react-relay";
import { format, formatDistanceToNow } from "date-fns";
import type { AwayPageQuery } from "./__generated__/AwayPageQuery.graphql";
import type { AwayPageSetModeMutation } from "./__generated__/AwayPageSetModeMutation.graphql";
import { Switch } from "./ui/switch";
import { cn } from "@/lib/utils";

const AwayQuery = graphql`
  query AwayPageQuery {
    modes {
      mode
      active
      ... on AwayMode {
        enabled
        window
        jitter
        minObservations
        lights {
          address
          deviceId
          name
          coverage
          currentTarget
          actions {
            at
            on
            slot
            onFraction
          }
        }
      }
    }
  }
`;

const SetModeMutation = graphql`
  mutation AwayPageSetModeMutation($active: Boolean!) {
    setMode(mode: AWAY, active: $active)
  }
`;

const SLOTS_PER_DAY = 48;

const TARGET_LABELS: Record<string, string> = {
  ON: "on",
  OFF: "off",
  LEAVE_ALONE: "left alone",
};

const TARGET_STYLES: Record<string, string> = {
  ON: "text-amber-600 dark:text-amber-400 border-amber-500/40",
  OFF: "text-muted-foreground border-border",
  LEAVE_ALONE: "text-muted-foreground border-border",
};

function Pill({ children, className }: React.PropsWithChildren<{ className?: string }>) {
  return (
    <span
      className={cn(
        "rounded-full border px-2 py-0.5 text-[10px] tracking-wide uppercase",
        className,
      )}
    >
      {children}
    </span>
  );
}

export default function AwayPage() {
  const [fetchKey, setFetchKey] = useState(0);
  const data = useLazyLoadQuery<AwayPageQuery>(
    AwayQuery,
    {},
    { fetchKey, fetchPolicy: "store-and-network" },
  );

  const [override, setOverride] = useState<boolean | null>(null);
  const [commit] = useMutation<AwayPageSetModeMutation>(SetModeMutation);

  const away = useMemo(
    () => data.modes.find((mode) => mode.mode === "AWAY"),
    [data.modes],
  );

  const active = override ?? away?.active ?? false;
  const lights = away?.lights ?? [];

  const toggle = () => {
    const desired = !active;
    setOverride(desired);
    commit({
      variables: { active: desired },
      onCompleted: () => {
        window.setTimeout(() => setFetchKey((key) => key + 1), 500);
      },
      onError: () => setOverride(!desired),
    });
  };

  if (!away) {
    return <p className="text-muted-foreground text-sm">Away mode is unavailable.</p>;
  }

  return (
    <div className="flex flex-col gap-10">
      <section className="bg-card border-border flex items-center justify-between gap-4 rounded-2xl border p-5">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium">Away mode</span>
            {!away.enabled && <Pill className="text-muted-foreground border-border">replay off</Pill>}
          </div>
          <p className="text-muted-foreground mt-1 text-xs">
            Replays {away.window} of light history, jittered by up to {away.jitter}, for
            slots with at least {away.minObservations} observations.
          </p>
        </div>
        <Switch checked={active} onCheckedChange={toggle} aria-label="Toggle away mode" />
      </section>

      <section>
        <h2 className="text-muted-foreground mb-4 text-xs font-semibold tracking-widest uppercase">
          Plan for the next 24 hours
        </h2>

        <div className="flex flex-col gap-3">
          {lights.map((light) => (
            <div
              key={light.address}
              className="bg-card border-border flex flex-col gap-3 rounded-2xl border p-4"
            >
              <div className="flex items-center justify-between gap-3">
                <div className="min-w-0">
                  <span className="truncate font-medium">{light.name}</span>
                  <span className="text-muted-foreground ml-2 font-mono text-xs">
                    {light.deviceId ?? light.address}
                  </span>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  <Pill className={TARGET_STYLES[light.currentTarget]}>
                    now {TARGET_LABELS[light.currentTarget]}
                  </Pill>
                  <Pill className="text-muted-foreground border-border">
                    {light.coverage}/{SLOTS_PER_DAY} slots
                  </Pill>
                </div>
              </div>

              {light.coverage === 0 ? (
                <p className="text-muted-foreground text-xs">
                  Not enough history yet — this light is left alone.
                </p>
              ) : light.actions.length === 0 ? (
                <p className="text-muted-foreground text-xs">
                  No switches planned in the next 24 hours.
                </p>
              ) : (
                <ol className="flex flex-wrap gap-2">
                  {light.actions.map((action) => (
                    <li
                      key={`${action.slot}-${action.at}`}
                      className={cn(
                        "border-border flex items-baseline gap-2 rounded-full border px-3 py-1",
                        action.on
                          ? "text-amber-600 dark:text-amber-400"
                          : "text-muted-foreground",
                      )}
                      title={`${Math.round(action.onFraction * 100)}% of past ${format(
                        new Date(action.at),
                        "EEEE",
                      )}s at this time`}
                    >
                      <span className="text-xs font-medium">
                        {format(new Date(action.at), "HH:mm")}
                      </span>
                      <span className="text-[10px] tracking-wide uppercase">
                        {action.on ? "on" : "off"}
                      </span>
                      <span className="text-muted-foreground text-[10px]">
                        {formatDistanceToNow(new Date(action.at), { addSuffix: true })}
                      </span>
                    </li>
                  ))}
                </ol>
              )}
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
