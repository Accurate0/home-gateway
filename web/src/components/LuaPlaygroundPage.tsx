import { useEffect, useRef, useState } from "react";
import {
  fetchQuery,
  graphql,
  useMutation,
  useRelayEnvironment,
} from "react-relay";
import type * as monaco from "monaco-editor";
import { formatDistanceToNow } from "date-fns";

import LuaEditor from "./LuaEditor";
import LuaApiExplorer from "./LuaApiExplorer";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import { indexLuaApi, type LuaApiIndex } from "@/lib/lua/api";
import { LUA_EXAMPLES } from "@/lib/lua/examples";
import {
  clearHistory,
  readHistory,
  recordHistory,
  type LuaHistoryEntry,
} from "@/lib/lua/history";
import type { LuaPlaygroundPageApiQuery } from "./__generated__/LuaPlaygroundPageApiQuery.graphql";
import type { LuaPlaygroundPageExecuteMutation } from "./__generated__/LuaPlaygroundPageExecuteMutation.graphql";

const ApiQuery = graphql`
  query LuaPlaygroundPageApiQuery {
    luaApi {
      definitions
      globals {
        name
        type
        optional
        declaration
      }
      aliases {
        name
        type
      }
      classes {
        name
        fields {
          name
          type
          optional
          declaration
        }
      }
      namespaces {
        name
        fields {
          name
          type
          optional
          declaration
        }
        functions {
          name
          returns
          scope
          signature
          params {
            name
            type
            optional
            declaration
          }
        }
      }
    }
  }
`;

const ExecuteMutation = graphql`
  mutation LuaPlaygroundPageExecuteMutation($script: String!, $dryRun: Boolean) {
    executeLua(script: $script, dryRun: $dryRun)
  }
`;

const BUTTON =
  "border-border text-muted-foreground hover:text-foreground cursor-pointer rounded-full border px-3 py-1 text-xs transition-colors disabled:cursor-default disabled:opacity-50";

type Outcome = {
  ok: boolean;
  body: string;
  elapsedMs: number;
};

export default function LuaPlaygroundPage() {
  const environment = useRelayEnvironment();
  const [index, setIndex] = useState<LuaApiIndex | null>(null);
  const [apiError, setApiError] = useState<string | null>(null);

  const [script, setScript] = useState(LUA_EXAMPLES[0].script);
  const [dryRun, setDryRun] = useState(true);
  const [outcome, setOutcome] = useState<Outcome | null>(null);
  const [running, setRunning] = useState(false);
  const [history, setHistory] = useState<LuaHistoryEntry[]>(readHistory);

  const editor = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const [commit] = useMutation<LuaPlaygroundPageExecuteMutation>(
    ExecuteMutation,
  );

  useEffect(() => {
    const subscription = fetchQuery<LuaPlaygroundPageApiQuery>(
      environment,
      ApiQuery,
      {},
    ).subscribe({
      next: (data) => setIndex(indexLuaApi(data.luaApi)),
      error: (error: Error) => setApiError(error.message),
    });

    return () => subscription.unsubscribe();
  }, [environment]);

  const run = () => {
    if (running || script.trim() === "") {
      return;
    }

    const current = script;
    const dry = dryRun;

    const started = performance.now();
    setRunning(true);

    const finish = (ok: boolean, body: string) => {
      setRunning(false);
      setOutcome({ ok, body, elapsedMs: performance.now() - started });
      setHistory(recordHistory({ script: current, dryRun: dry, ok }));
    };

    commit({
      variables: { script: current, dryRun: dry },
      onCompleted: (response, errors) => {
        if (errors && errors.length > 0) {
          finish(false, errors.map((error) => error.message).join("\n"));
          return;
        }

        finish(true, JSON.stringify(response.executeLua, null, 2));
      },
      onError: (error) => finish(false, error.message),
    });
  };

  const latestRun = useRef(run);

  useEffect(() => {
    latestRun.current = run;
  });

  const insert = (text: string) => {
    const instance = editor.current;

    if (!instance) {
      setScript((current) => `${current}\n${text}`);
      return;
    }

    const selection = instance.getSelection();

    if (selection) {
      instance.executeEdits("lua-explorer", [
        { range: selection, text, forceMoveMarkers: true },
      ]);
    }

    instance.focus();
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-3">
        <button
          type="button"
          onClick={run}
          disabled={running}
          className={cn(
            BUTTON,
            "border-foreground/30 text-foreground px-4 font-medium",
          )}
        >
          {running ? "Running…" : "Run ⌘⏎"}
        </button>

        <label className="flex cursor-pointer items-center gap-2 text-xs">
          <Switch checked={dryRun} onCheckedChange={setDryRun} />
          <span className={dryRun ? "text-foreground" : "text-muted-foreground"}>
            dry run
          </span>
        </label>

        <span className="text-muted-foreground text-xs">
          Runs with your own scopes — namespaces you cannot use are not exposed.
        </span>

        <div className="ml-auto flex flex-wrap gap-1">
          {LUA_EXAMPLES.map((example) => (
            <button
              key={example.name}
              type="button"
              title={example.description}
              onClick={() => setScript(example.script)}
              className={BUTTON}
            >
              {example.name}
            </button>
          ))}
        </div>
      </div>

      <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_20rem]">
        <div className="flex min-w-0 flex-col gap-4">
          <div className="bg-card border-border h-[26rem] overflow-hidden rounded-2xl border">
            <LuaEditor
              value={script}
              onChange={setScript}
              index={index}
              onRun={() => latestRun.current()}
              onReady={(instance) => {
                editor.current = instance;
              }}
            />
          </div>

          <section className="bg-card border-border rounded-2xl border">
            <header className="border-border flex items-center gap-3 border-b px-4 py-3">
              <h2 className="text-muted-foreground text-xs font-semibold tracking-widest uppercase">
                Result
              </h2>
              {outcome && (
                <span
                  className={cn(
                    "rounded-full border px-2 py-0.5 text-[10px] tracking-wide uppercase",
                    outcome.ok
                      ? "border-emerald-500/40 text-emerald-600 dark:text-emerald-400"
                      : "border-red-500/40 text-red-600 dark:text-red-400",
                  )}
                >
                  {outcome.ok ? "ok" : "error"}
                </span>
              )}
              {outcome && (
                <span className="text-muted-foreground text-[10px]">
                  {Math.round(outcome.elapsedMs)} ms
                </span>
              )}
            </header>

            <pre className="max-h-72 overflow-auto p-4 font-mono text-[11px] leading-relaxed whitespace-pre-wrap">
              {outcome?.body ?? "Run a script to see its result."}
            </pre>
          </section>
        </div>

        <div className="flex min-h-0 flex-col gap-4">
          {index ? (
            <LuaApiExplorer index={index} onInsert={insert} />
          ) : (
            <section className="bg-card border-border rounded-2xl border p-4">
              <p className="text-muted-foreground text-xs">
                {apiError
                  ? `API reference unavailable: ${apiError}`
                  : "Loading the API reference…"}
              </p>
            </section>
          )}

          <section className="bg-card border-border rounded-2xl border">
            <header className="border-border flex items-center justify-between border-b px-4 py-3">
              <h2 className="text-muted-foreground text-xs font-semibold tracking-widest uppercase">
                History
              </h2>
              {history.length > 0 && (
                <button
                  type="button"
                  onClick={() => setHistory(clearHistory())}
                  className="text-muted-foreground hover:text-foreground cursor-pointer text-xs"
                >
                  clear
                </button>
              )}
            </header>

            <ul className="max-h-64 overflow-auto p-2">
              {history.length === 0 && (
                <li className="text-muted-foreground px-2 py-2 text-xs">
                  Scripts you run are kept in this browser.
                </li>
              )}

              {history.map((entry) => (
                <li key={entry.id}>
                  <button
                    type="button"
                    onClick={() => setScript(entry.script)}
                    className="hover:bg-muted w-full cursor-pointer rounded-md px-2 py-1.5 text-left"
                  >
                    <span className="block truncate font-mono text-[11px]">
                      {entry.script.split("\n")[0]}
                    </span>
                    <span className="text-muted-foreground text-[10px]">
                      {formatDistanceToNow(new Date(entry.at), {
                        addSuffix: true,
                      })}
                      {entry.dryRun && " · dry run"}
                      {!entry.ok && " · failed"}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </section>
        </div>
      </div>
    </div>
  );
}
