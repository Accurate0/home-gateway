import {
  DoorClosed,
  DoorOpen,
  Lightbulb,
  LightbulbOff,
  PersonStanding,
  Thermometer,
  UserX,
} from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { Entity } from "@/entities";

function StateBadge({
  value,
  labels,
}: {
  value: boolean | null | undefined;
  labels: { on: string; off: string };
}) {
  if (value == null) {
    return <Badge variant="outline">unknown</Badge>;
  }
  return (
    <Badge variant={value ? "default" : "secondary"}>
      {value ? labels.on : labels.off}
    </Badge>
  );
}

function fmt(value: number | null | undefined, unit: string, digits = 1) {
  return value == null ? "—" : `${value.toFixed(digits)}${unit}`;
}

function EntityIcon({ entity }: { entity: Entity }) {
  const className = "text-muted-foreground mt-0.5 size-5 shrink-0";
  switch (entity.kind) {
    case "light":
      return entity.on ? (
        <Lightbulb className={className} />
      ) : (
        <LightbulbOff className={className} />
      );
    case "door":
      return entity.open ? (
        <DoorOpen className={className} />
      ) : (
        <DoorClosed className={className} />
      );
    case "presence":
      return entity.present ? (
        <PersonStanding className={className} />
      ) : (
        <UserX className={className} />
      );
    case "environment":
      return <Thermometer className={className} />;
  }
}

export default function EntityCard({ entity }: { entity: Entity }) {
  return (
    <Card className="gap-3 py-4">
      <CardContent className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-3">
          <EntityIcon entity={entity} />
          <div>
            <div className="font-medium leading-tight">{entity.name}</div>
            <div className="text-muted-foreground text-xs">{entity.id}</div>
          </div>
        </div>
        <div className="text-right">
          {entity.kind === "light" && (
            <StateBadge value={entity.on} labels={{ on: "on", off: "off" }} />
          )}
          {entity.kind === "door" && (
            <StateBadge
              value={entity.open}
              labels={{ on: "open", off: "closed" }}
            />
          )}
          {entity.kind === "presence" && (
            <StateBadge
              value={entity.present}
              labels={{ on: "present", off: "away" }}
            />
          )}
          {entity.kind === "environment" && (
            <div className="text-sm">
              <div className="text-lg font-semibold">
                {fmt(entity.temperature, "°")}
              </div>
              <div className="text-muted-foreground">
                {fmt(entity.humidity, "%", 0)} hum
              </div>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
