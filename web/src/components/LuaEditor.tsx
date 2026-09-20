import { useEffect, useRef } from "react";
import Editor, { loader } from "@monaco-editor/react";
import * as monaco from "monaco-editor";

import "@/lib/lua/workers";
import { registerLuaApi } from "@/lib/lua/monaco";
import type { LuaApiIndex } from "@/lib/lua/api";

loader.config({ monaco });

const THEMES: Record<"light" | "dark", monaco.editor.IStandaloneThemeData> = {
  light: {
    base: "vs",
    inherit: true,
    rules: [],
    colors: {
      "editor.background": "#fdfcfa",
      "editorLineNumber.foreground": "#b4aca1",
      "editor.lineHighlightBackground": "#f3efe9",
    },
  },
  dark: {
    base: "vs-dark",
    inherit: true,
    rules: [],
    colors: {
      "editor.background": "#1c1a17",
      "editorLineNumber.foreground": "#6b645b",
      "editor.lineHighlightBackground": "#26231f",
    },
  },
};

const OPTIONS: monaco.editor.IStandaloneEditorConstructionOptions = {
  fontSize: 13,
  fontLigatures: true,
  minimap: { enabled: false },
  scrollBeyondLastLine: false,
  renderLineHighlight: "line",
  padding: { top: 12, bottom: 12 },
  tabSize: 2,
  automaticLayout: true,
  scrollbar: { verticalScrollbarSize: 8, horizontalScrollbarSize: 8 },
};

function theme(): keyof typeof THEMES {
  return document.documentElement.classList.contains("dark") ? "dark" : "light";
}

type Props = {
  value: string;
  onChange: (value: string) => void;
  index: LuaApiIndex | null;
  onRun: () => void;
  onReady: (editor: monaco.editor.IStandaloneCodeEditor) => void;
};

export default function LuaEditor({
  value,
  onChange,
  index,
  onRun,
  onReady,
}: Props) {
  const run = useRef(onRun);

  useEffect(() => {
    run.current = onRun;
  }, [onRun]);

  useEffect(() => {
    if (!index) {
      return;
    }

    const registration = registerLuaApi(monaco, index);
    return () => registration.dispose();
  }, [index]);

  const mount = (editor: monaco.editor.IStandaloneCodeEditor) => {
    monaco.editor.defineTheme("home-gateway-light", THEMES.light);
    monaco.editor.defineTheme("home-gateway-dark", THEMES.dark);
    monaco.editor.setTheme(`home-gateway-${theme()}`);

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () =>
      run.current(),
    );

    onReady(editor);
  };

  return (
    <Editor
      language="lua"
      value={value}
      options={OPTIONS}
      onChange={(next) => onChange(next ?? "")}
      onMount={mount}
      loading={<span className="text-muted-foreground text-sm">Loading…</span>}
    />
  );
}
