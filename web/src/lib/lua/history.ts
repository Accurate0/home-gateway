import { readStored, removeStored, writeStored } from "@/lib/storage";

const KEY = "home-gateway.lua.history";
const LIMIT = 25;

export type LuaHistoryEntry = {
  id: string;
  script: string;
  dryRun: boolean;
  ok: boolean;
  at: string;
};

function identifier(): string {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
}

export function readHistory(): LuaHistoryEntry[] {
  const entries = readStored<LuaHistoryEntry[]>(KEY, []);

  return Array.isArray(entries) ? entries : [];
}

export function recordHistory(
  entry: Omit<LuaHistoryEntry, "id" | "at">,
): LuaHistoryEntry[] {
  const next = [
    { ...entry, id: identifier(), at: new Date().toISOString() },
    ...readHistory().filter((previous) => previous.script !== entry.script),
  ].slice(0, LIMIT);

  writeStored(KEY, next);

  return next;
}

export function clearHistory(): LuaHistoryEntry[] {
  removeStored(KEY);

  return [];
}
