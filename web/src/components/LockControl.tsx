import { Lock, LockOpen } from "lucide-react";
import { cn } from "@/lib/utils";
import { activeLock, formatLockRemaining, type Entity } from "@/entities";

export interface LockActions {
  onLock: (durationSeconds: number | null) => void;
  onUnlock: () => void;
}

const HOUR = 60 * 60;

const LOCK_DURATIONS: {
  label: string;
  title: string;
  seconds: number | null;
}[] = [
  { label: "1h", title: "Lock for 1 hour", seconds: HOUR },
  { label: "6h", title: "Lock for 6 hours", seconds: 6 * HOUR },
  { label: "Hold", title: "Lock until unlocked", seconds: null },
];

const CHIP =
  "flex flex-1 items-center justify-center rounded-lg border py-1.5 text-xs font-medium";
const IDLE = "border-border text-muted-foreground hover:bg-muted";

export default function LockControl({
  entity,
  actions,
  now,
}: {
  entity: Entity;
  actions?: LockActions;
  now: number;
}) {
  if (entity.lock === undefined) return null;

  const lock = activeLock(entity, now);

  if (lock) {
    return (
      <div className="flex items-center gap-2">
        <div
          className="flex min-w-0 flex-1 items-center gap-1.5 text-xs"
          title="Workflows can't change this device while it is locked"
        >
          <Lock className="size-3.5 shrink-0" strokeWidth={2} />
          <span className="truncate">
            <span className="font-medium capitalize">{lock.source}</span>
            <span className="text-muted-foreground">
              {" · "}
              {formatLockRemaining(lock, now)}
            </span>
          </span>
        </div>
        <button
          type="button"
          onClick={actions?.onUnlock}
          className={cn(CHIP, IDLE, "flex-none px-2.5")}
        >
          <LockOpen className="mr-1.5 size-3.5" strokeWidth={2} />
          Unlock
        </button>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2">
      <span
        className="text-muted-foreground flex items-center gap-1 text-xs"
        title="Stop workflows from changing this device"
      >
        <Lock className="size-3.5" strokeWidth={2} />
        Lock
      </span>
      {LOCK_DURATIONS.map((duration) => (
        <button
          key={duration.label}
          type="button"
          title={duration.title}
          onClick={() => actions?.onLock(duration.seconds)}
          className={cn(CHIP, IDLE)}
        >
          {duration.label}
        </button>
      ))}
    </div>
  );
}
