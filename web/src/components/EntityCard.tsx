import {
  BatteryFull,
  BatteryLow,
  BatteryMedium,
  DoorClosed,
  DoorOpen,
  Lightbulb,
  LightbulbOff,
  MonitorSmartphone,
  Palette,
  PersonStanding,
  SlidersHorizontal,
  Thermometer,
  UserX,
} from "lucide-react";
import { useState } from "react";
import { Popover, Slider } from "radix-ui";
import { cn } from "@/lib/utils";
import { formatLastSeen, type Entity } from "@/entities";

export interface LightActions {
  onToggle: () => void;
  onSetBrightness: (value: number) => void;
  onColourMove: (value: number) => void;
  onSetColour: (hex: string) => void;
  canSetColour: boolean;
}

const COLOUR_SWATCHES = [
  "#ff5a5a",
  "#ff9d3c",
  "#ffd23c",
  "#3ce17a",
  "#3cc7ff",
  "#6a7bff",
  "#c86bff",
  "#ffffff",
];

const BRIGHTNESS_MAX = 254;
const COLOUR_MOVE_RATE = 40;

function fmt(value: number | null | undefined, unit: string, digits = 1) {
  return value == null ? "—" : `${value.toFixed(digits)}${unit}`;
}

function StatePill({
  tone,
  children,
}: {
  tone: "on" | "off" | "unknown";
  children: React.ReactNode;
}) {
  return (
    <span
      className={cn(
        "rounded-full px-2.5 py-0.5 text-xs font-semibold tracking-wide uppercase",
        tone === "on" && "bg-foreground text-background",
        tone === "off" && "bg-muted text-muted-foreground",
        tone === "unknown" &&
          "border-border text-muted-foreground border bg-transparent",
      )}
    >
      {children}
    </span>
  );
}

function LastSeen({
  entity,
  now,
  className,
}: {
  entity: Entity;
  now: number;
  className?: string;
}) {
  const label = formatLastSeen(entity.lastSeen ?? entity.time, now);
  if (!label) return null;
  return (
    <span
      className={cn(
        "text-muted-foreground text-[11px] tabular-nums",
        className,
      )}
      title="Last seen"
    >
      <span aria-hidden className="mr-1">
        -
      </span>
      {label}
    </span>
  );
}

function Tile({
  className,
  ...props
}: React.ComponentProps<"div">) {
  return (
    <div
      className={cn(
        "bg-card border-border flex flex-col rounded-2xl border p-4 transition-all",
        className,
      )}
      {...props}
    />
  );
}

function LightControls({ actions }: { actions: LightActions }) {
  const [brightness, setBrightness] = useState(Math.round(BRIGHTNESS_MAX / 2));
  const pct = Math.round((brightness / BRIGHTNESS_MAX) * 100);

  const holdMove = (value: number) => ({
    onPointerDown: () => actions.onColourMove(value),
    onPointerUp: () => actions.onColourMove(0),
    onPointerLeave: () => actions.onColourMove(0),
  });

  return (
    <Popover.Portal>
      <Popover.Content
        align="end"
        sideOffset={8}
        onClick={(e) => e.stopPropagation()}
        className="bg-popover text-popover-foreground border-border z-50 w-64 rounded-2xl border p-4 shadow-lg outline-none"
      >
        <div className="mb-4 flex gap-2">
          <button
            onClick={actions.onToggle}
            className="bg-foreground text-background flex-1 rounded-lg py-1.5 text-sm font-medium"
          >
            Toggle
          </button>
        </div>

        <div className="mb-4">
          <div className="text-muted-foreground mb-2 flex justify-between text-xs">
            <span className="uppercase tracking-wide">Brightness</span>
            <span>{pct}%</span>
          </div>
          <Slider.Root
            value={[brightness]}
            min={0}
            max={BRIGHTNESS_MAX}
            step={1}
            onValueChange={([v]) => setBrightness(v)}
            onValueCommit={([v]) => actions.onSetBrightness(v)}
            className="relative flex h-4 w-full touch-none items-center select-none"
          >
            <Slider.Track className="bg-muted relative h-1.5 grow rounded-full">
              <Slider.Range className="bg-state-light absolute h-full rounded-full" />
            </Slider.Track>
            <Slider.Thumb className="border-state-light bg-background block size-4 rounded-full border-2 shadow-sm outline-none" />
          </Slider.Root>
        </div>

        <div>
          <div className="text-muted-foreground mb-2 text-xs uppercase tracking-wide">
            Colour temperature
          </div>
          <div className="flex gap-2">
            <button
              {...holdMove(COLOUR_MOVE_RATE)}
              className="border-border hover:bg-accent flex-1 rounded-lg border py-1.5 text-sm"
            >
              Warmer
            </button>
            <button
              {...holdMove(-COLOUR_MOVE_RATE)}
              className="border-border hover:bg-accent flex-1 rounded-lg border py-1.5 text-sm"
            >
              Cooler
            </button>
          </div>
          <div className="text-muted-foreground mt-1.5 text-[11px]">
            Hold to shift, release to stop
          </div>
        </div>

        {actions.canSetColour && (
          <div className="mt-4">
            <div className="text-muted-foreground mb-2 text-xs uppercase tracking-wide">
              Colour
            </div>
            <div className="flex flex-wrap items-center gap-2">
              {COLOUR_SWATCHES.map((hex) => (
                <button
                  key={hex}
                  aria-label={hex}
                  onClick={() => actions.onSetColour(hex)}
                  style={{ backgroundColor: hex }}
                  className="border-border/60 size-6 rounded-full border transition-transform hover:scale-110"
                />
              ))}
              <label className="border-border ml-auto grid size-6 cursor-pointer place-items-center rounded-full border">
                <Palette className="text-muted-foreground size-3.5" />
                <input
                  type="color"
                  onChange={(e) => actions.onSetColour(e.target.value)}
                  className="absolute size-0 opacity-0"
                />
              </label>
            </div>
          </div>
        )}
      </Popover.Content>
    </Popover.Portal>
  );
}

function LightTile({
  entity,
  actions,
  now,
}: {
  entity: Entity;
  actions?: LightActions;
  now: number;
}) {
  const on = entity.on;
  const unknown = on == null;
  return (
    <Popover.Root>
      <Tile
        role="button"
        tabIndex={0}
        onClick={actions?.onToggle}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            actions?.onToggle();
          }
        }}
        className={cn(
          "col-span-1 cursor-pointer justify-between gap-3 select-none",
          "hover:-translate-y-0.5 active:translate-y-0",
          on
            ? "border-state-light/50 bg-state-light/15 ring-state-light/40 shadow-[0_8px_30px_-12px_var(--state-light)] ring-1"
            : "hover:border-foreground/20",
        )}
      >
        <div className="flex items-start justify-between">
          <div
            className={cn(
              "grid size-10 place-items-center rounded-xl transition-colors",
              on
                ? "bg-state-light text-state-light-foreground"
                : "bg-muted text-muted-foreground",
            )}
          >
            {on ? (
              <Lightbulb
                className="size-5"
                fill="currentColor"
                strokeWidth={1.5}
              />
            ) : (
              <LightbulbOff className="size-5" strokeWidth={1.5} />
            )}
          </div>
          {actions && (
            <Popover.Trigger asChild>
              <button
                aria-label="Light controls"
                onClick={(e) => e.stopPropagation()}
                className="text-muted-foreground hover:bg-accent hover:text-foreground -m-1 grid size-8 place-items-center rounded-lg transition-colors"
              >
                <SlidersHorizontal className="size-4" strokeWidth={1.75} />
              </button>
            </Popover.Trigger>
          )}
        </div>
        <div>
          <div className="leading-tight font-medium">{entity.name}</div>
          <div className="text-muted-foreground mb-2 flex items-center gap-1 text-xs">
            <span>{entity.id}</span>
            <LastSeen entity={entity} now={now} />
          </div>
          <StatePill tone={unknown ? "unknown" : on ? "on" : "off"}>
            {unknown ? "unknown" : on ? "on" : "off"}
          </StatePill>
        </div>
      </Tile>
      {actions && <LightControls actions={actions} />}
    </Popover.Root>
  );
}

const ENVIRONMENT_METRICS: {
  label: string;
  field: keyof Entity;
  capability: string;
  unit: string;
  digits?: number;
}[] = [
  { label: "Temperature", field: "temperature", capability: "TEMPERATURE", unit: "°C" },
  { label: "Humidity", field: "humidity", capability: "HUMIDITY", unit: "%", digits: 0 },
  { label: "Pressure", field: "pressure", capability: "PRESSURE", unit: " hPa", digits: 0 },
  { label: "Illuminance", field: "lux", capability: "LUX", unit: " lx", digits: 0 },
  { label: "UV index", field: "uvIndex", capability: "UV_INDEX", unit: "" },
];

function environmentMetrics(entity: Entity) {
  const caps = entity.capabilities;
  if (!caps || caps.length === 0) return ENVIRONMENT_METRICS;
  return ENVIRONMENT_METRICS.filter((m) => caps.includes(m.capability));
}

function EnvironmentDetails({ entity, now }: { entity: Entity; now: number }) {
  return (
    <Popover.Portal>
      <Popover.Content
        align="end"
        sideOffset={8}
        onClick={(e) => e.stopPropagation()}
        className="bg-popover text-popover-foreground border-border z-50 w-64 rounded-2xl border p-4 shadow-lg outline-none"
      >
        <div className="mb-3 flex items-center justify-between">
          <span className="font-medium">{entity.name}</span>
          <LastSeen entity={entity} now={now} />
        </div>
        <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
          {environmentMetrics(entity).map(({ label, field, unit, digits }) => (
            <div key={label} className="flex flex-col">
              <dt className="text-muted-foreground text-xs uppercase tracking-wide">
                {label}
              </dt>
              <dd className="tabular-nums">
                {fmt(entity[field] as number | null | undefined, unit, digits)}
              </dd>
            </div>
          ))}
        </dl>
      </Popover.Content>
    </Popover.Portal>
  );
}

function EnvironmentTile({ entity, now }: { entity: Entity; now: number }) {
  const hum = entity.humidity;
  const pct = hum == null ? 0 : Math.max(0, Math.min(100, hum));
  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <Tile
          role="button"
          tabIndex={0}
          className="col-span-2 cursor-pointer justify-between transition-all select-none hover:-translate-y-0.5 hover:border-foreground/20 active:translate-y-0 sm:col-span-1 lg:col-span-2"
        >
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-2.5">
              <div className="bg-muted text-muted-foreground grid size-9 place-items-center rounded-xl">
                <Thermometer className="size-5" strokeWidth={1.5} />
              </div>
              <div>
                <div className="leading-tight font-medium">{entity.name}</div>
                <div className="text-muted-foreground flex items-center gap-1 text-xs">
                  <span>{entity.id}</span>
                  <LastSeen entity={entity} now={now} />
                </div>
              </div>
            </div>
            <div className="font-display text-3xl font-semibold tracking-tight">
              {fmt(entity.temperature, "°")}
            </div>
          </div>
          <div className="mt-4">
            <div className="text-muted-foreground mb-1.5 flex justify-between text-xs">
              <span>Humidity</span>
              <span>{fmt(hum, "%", 0)}</span>
            </div>
            <div className="bg-muted h-1.5 overflow-hidden rounded-full">
              <div
                className="bg-state-present/70 h-full rounded-full transition-[width]"
                style={{ width: `${pct}%` }}
              />
            </div>
          </div>
        </Tile>
      </Popover.Trigger>
      <EnvironmentDetails entity={entity} now={now} />
    </Popover.Root>
  );
}

const STATUS_TONES = {
  present: {
    active:
      "border-state-present/40 bg-state-present/10",
    iconActive: "bg-state-present text-state-present-foreground",
  },
  open: {
    active: "border-state-open/40 bg-state-open/10",
    iconActive: "bg-state-open text-state-open-foreground",
  },
} as const;

function StatusTile({
  entity,
  active,
  icon,
  activeIcon,
  labels,
  tone,
  now,
}: {
  entity: Entity;
  active: boolean | null | undefined;
  icon: React.ReactNode;
  activeIcon: React.ReactNode;
  labels: { on: string; off: string };
  tone: keyof typeof STATUS_TONES;
  now: number;
}) {
  const unknown = active == null;
  const t = STATUS_TONES[tone];
  return (
    <Tile
      className={cn("col-span-1 justify-between gap-3", active && t.active)}
    >
      <div
        className={cn(
          "grid size-10 place-items-center rounded-xl transition-colors",
          active ? t.iconActive : "bg-muted text-muted-foreground",
        )}
      >
        {active ? activeIcon : icon}
      </div>
      <div>
        <div className="leading-tight font-medium">{entity.name}</div>
        <div className="text-muted-foreground mb-2 flex items-center gap-1 text-xs">
          <span>{entity.id}</span>
          <LastSeen entity={entity} now={now} />
        </div>
        <StatePill tone={unknown ? "unknown" : active ? "on" : "off"}>
          {unknown ? "unknown" : active ? labels.on : labels.off}
        </StatePill>
      </div>
    </Tile>
  );
}

function EinkDisplayTile({ entity, now }: { entity: Entity; now: number }) {
  const voltage = entity.batteryVoltage;
  const percentage = entity.batteryPercentage;
  const BatteryIcon =
    percentage == null || percentage >= 60
      ? BatteryFull
      : percentage >= 25
        ? BatteryMedium
        : BatteryLow;
  return (
    <Tile className="col-span-1 justify-between gap-3">
      <div className="flex items-start justify-between">
        <div className="bg-muted text-muted-foreground grid size-10 place-items-center rounded-xl">
          <MonitorSmartphone className="size-5" strokeWidth={1.5} />
        </div>
        <div className="text-muted-foreground flex items-center gap-1.5 tabular-nums">
          <BatteryIcon className="size-4" strokeWidth={1.75} />
          <div className="flex flex-col items-end leading-tight">
            <span className="text-sm">
              {percentage == null ? "—" : `${Math.round(percentage)}%`}
            </span>
            {voltage != null && (
              <span className="text-muted-foreground/70 text-xs">
                {fmt(voltage, " V", 2)}
              </span>
            )}
          </div>
        </div>
      </div>
      <div>
        <div className="leading-tight font-medium">{entity.name}</div>
        <div className="text-muted-foreground flex items-center gap-1 text-xs">
          <span>{entity.id}</span>
          <LastSeen entity={entity} now={now} />
        </div>
      </div>
    </Tile>
  );
}

export default function EntityCard({
  entity,
  lightActions,
  now,
}: {
  entity: Entity;
  lightActions?: LightActions;
  now: number;
}) {
  switch (entity.kind) {
    case "light":
      return <LightTile entity={entity} actions={lightActions} now={now} />;
    case "environment":
      return <EnvironmentTile entity={entity} now={now} />;
    case "door":
      return (
        <StatusTile
          entity={entity}
          active={entity.open}
          icon={<DoorClosed className="size-5" strokeWidth={1.5} />}
          activeIcon={<DoorOpen className="size-5" strokeWidth={1.5} />}
          labels={{ on: "open", off: "closed" }}
          tone="open"
          now={now}
        />
      );
    case "presence":
      return (
        <StatusTile
          entity={entity}
          active={entity.present}
          icon={<UserX className="size-5" strokeWidth={1.5} />}
          activeIcon={<PersonStanding className="size-5" strokeWidth={1.5} />}
          labels={{ on: "present", off: "away" }}
          tone="present"
          now={now}
        />
      );
    case "einkDisplay":
      return <EinkDisplayTile entity={entity} now={now} />;
  }
}
