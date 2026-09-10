import { useState } from "react";
import { graphql, useLazyLoadQuery, useMutation } from "react-relay";
import { format, formatDistanceToNow } from "date-fns";
import type { ModesPageQuery } from "./__generated__/ModesPageQuery.graphql";
import type { ModesPageSetModeMutation } from "./__generated__/ModesPageSetModeMutation.graphql";
import { cn } from "@/lib/utils";

const ModesQuery = graphql`
  query ModesPageQuery {
    mode {
      active
      node(mode: VACATION) {
        mode
        ... on VacationMode {
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
  }
`;

const SetModeMutation = graphql`
  mutation ModesPageSetModeMutation($mode: Mode!) {
    setMode(mode: $mode)
  }
`;

type ModeValue = ModesPageSetModeMutation["variables"]["mode"];

const MODES: { value: ModeValue; label: string }[] = [
  { value: "HOME", label: "Home" },
  { value: "AWAY", label: "Away" },
  { value: "VACATION", label: "Vacation" },
  { value: "GUEST", label: "Guest" },
];

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

export default function ModesPage() {
  const [fetchKey, setFetchKey] = useState(0);
  const data = useLazyLoadQuery<ModesPageQuery>(
    ModesQuery,
    {},
    { fetchKey, fetchPolicy: "store-and-network" },
  );

  const [override, setOverride] = useState<ModeValue | null>(null);
  const [commit] = useMutation<ModesPageSetModeMutation>(SetModeMutation);

  const active = override ?? data.mode.active;
  const vacation = data.mode.node;
  const lights = vacation.lights ?? [];

  const select = (mode: ModeValue) => {
    if (mode === active) {
      return;
    }

    setOverride(mode);
    commit({
      variables: { mode },
      onCompleted: () => {
        window.setTimeout(() => setFetchKey((key) => key + 1), 500);
      },
      onError: () => setOverride(null),
    });
  };

  return (
    <div className="flex flex-col gap-10">
      <section className="bg-card border-border flex flex-col gap-4 rounded-2xl border p-5 sm:flex-row sm:items-center sm:justify-between">
        <div className="min-w-0">
          <span className="font-medium">House mode</span>
          <p className="text-muted-foreground mt-1 text-xs">
            Exactly one mode is active. Workflows limited to other modes will not fire.
          </p>
        </div>

        <div role="group" aria-label="House mode" className="border-border flex rounded-md border text-xs">
          {MODES.map(({ value, label }) => (
            <button
              key={value}
              type="button"
              onClick={() => select(value)}
              aria-pressed={active === value}
              className={cn(
                "cursor-pointer px-3 py-1.5 first:rounded-l-md last:rounded-r-md",
                active === value
                  ? "bg-muted text-foreground font-medium"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              {label}
            </button>
          ))}
        </div>
      </section>

      <section>
        <div className="mb-2 flex items-center gap-2">
          <h2 className="text-muted-foreground text-xs font-semibold tracking-widest uppercase">
            Vacation replay
          </h2>
          {active === "VACATION" && (
            <Pill className="border-amber-500/40 text-amber-600 dark:text-amber-400">replaying</Pill>
          )}
          {vacation.enabled === false && (
            <Pill className="text-muted-foreground border-border">replay off</Pill>
          )}
        </div>

        <p className="text-muted-foreground mb-4 text-xs">
          Replays {vacation.window} of light history, jittered by up to {vacation.jitter}, for
          slots with at least {vacation.minObservations} observations. Plan for the next 24 hours:
        </p>

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
