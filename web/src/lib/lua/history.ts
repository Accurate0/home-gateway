const KEY = "home-gateway.lua.history";
const LIMIT = 25;

export type LuaHistoryEntry = {
  id: string;
  script: string;
  dryRun: boolean;
  ok: boolean;
  at: string;
};

export function readHistory(): LuaHistoryEntry[] {
  const raw = window.localStorage.getItem(KEY);

  if (!raw) {
    return [];
  }

  try {
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed) ? (parsed as LuaHistoryEntry[]) : [];
  } catch {
    return [];
  }
}

export function recordHistory(
  entry: Omit<LuaHistoryEntry, "id" | "at">,
): LuaHistoryEntry[] {
  const next = [
    { ...entry, id: crypto.randomUUID(), at: new Date().toISOString() },
    ...readHistory().filter((previous) => previous.script !== entry.script),
  ].slice(0, LIMIT);

  window.localStorage.setItem(KEY, JSON.stringify(next));

  return next;
}

export function clearHistory(): LuaHistoryEntry[] {
  window.localStorage.removeItem(KEY);

  return [];
}
