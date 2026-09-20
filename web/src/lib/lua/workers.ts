import type * as Monaco from "monaco-editor";
import EditorWorker from "monaco-editor/esm/vs/editor/editor.worker.js?worker";

const host = globalThis as { MonacoEnvironment?: Monaco.Environment };

if (!host.MonacoEnvironment) {
  host.MonacoEnvironment = { getWorker: () => new EditorWorker() };
}
