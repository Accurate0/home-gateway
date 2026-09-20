import { useMemo, useState } from "react";

import { cn } from "@/lib/utils";
import {
  callText,
  functionDetail,
  type LuaApiIndex,
  type LuaFunction,
  type LuaNamespace,
} from "@/lib/lua/api";

type Props = {
  index: LuaApiIndex;
  onInsert: (text: string) => void;
};

function matches(namespace: LuaNamespace, fn: LuaFunction, needle: string) {
  return (
    needle === "" ||
    `${namespace.name}.${fn.name}`.toLowerCase().includes(needle) ||
    (fn.scope ?? "").toLowerCase().includes(needle)
  );
}

export default function LuaApiExplorer({ index, onInsert }: Props) {
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState<string | null>(null);
  const [types, setTypes] = useState(false);

  const needle = query.trim().toLowerCase();

  const namespaces = useMemo(
    () =>
      index.api.namespaces
        .map((namespace) => ({
          namespace,
          functions: namespace.functions.filter((fn) =>
            matches(namespace, fn, needle),
          ),
        }))
        .filter((entry) => entry.functions.length > 0),
    [index, needle],
  );

  if (types) {
    return (
      <section className="bg-card border-border flex min-h-0 flex-col rounded-2xl border">
        <header className="border-border flex items-center justify-between gap-2 border-b px-4 py-3">
          <h2 className="text-muted-foreground text-xs font-semibold tracking-widest uppercase">
            Type definitions
          </h2>
          <button
            type="button"
            onClick={() => setTypes(false)}
            className="text-muted-foreground hover:text-foreground cursor-pointer text-xs"
          >
            back
          </button>
        </header>
        <pre className="min-h-0 flex-1 overflow-auto p-4 font-mono text-[11px] leading-relaxed">
          {index.api.definitions}
        </pre>
      </section>
    );
  }

  return (
    <section className="bg-card border-border flex min-h-0 flex-col rounded-2xl border">
      <header className="border-border flex flex-col gap-3 border-b px-4 py-3">
        <div className="flex items-center justify-between gap-2">
          <h2 className="text-muted-foreground text-xs font-semibold tracking-widest uppercase">
            API
          </h2>
          <button
            type="button"
            onClick={() => setTypes(true)}
            className="text-muted-foreground hover:text-foreground cursor-pointer text-xs"
          >
            definitions
          </button>
        </div>
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search functions or scopes…"
          className="border-border focus-visible:ring-ring/50 rounded-md border bg-transparent px-2 py-1 text-xs outline-none focus-visible:ring-[3px]"
        />
      </header>

      <div className="min-h-0 flex-1 overflow-auto px-2 py-2">
        {namespaces.length === 0 && (
          <p className="text-muted-foreground px-2 py-4 text-xs">
            Nothing matches “{query}”.
          </p>
        )}

        {namespaces.map(({ namespace, functions }) => {
          const expanded = needle !== "" || open === namespace.name;

          return (
            <div key={namespace.name} className="mb-1">
              <button
                type="button"
                onClick={() =>
                  setOpen(open === namespace.name ? null : namespace.name)
                }
                className={cn(
                  "hover:bg-muted flex w-full cursor-pointer items-baseline justify-between rounded-md px-2 py-1.5 text-left",
                  expanded && "text-foreground",
                )}
              >
                <span className="font-mono text-xs font-medium">
                  {namespace.name}
                </span>
                <span className="text-muted-foreground text-[10px]">
                  {functions.length}
                </span>
              </button>

              {expanded && (
                <ul className="border-border ml-3 border-l pl-2">
                  {functions.map((fn) => (
                    <li key={fn.name}>
                      <button
                        type="button"
                        onClick={() => onInsert(callText(namespace.name, fn))}
                        title="Insert at the cursor"
                        className="hover:bg-muted w-full cursor-pointer rounded-md px-2 py-1 text-left"
                      >
                        <span className="block font-mono text-[11px] break-all">
                          {functionDetail(fn)}
                        </span>
                        {fn.scope && (
                          <span className="text-muted-foreground text-[10px]">
                            {fn.scope}
                          </span>
                        )}
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          );
        })}
      </div>
    </section>
  );
}
