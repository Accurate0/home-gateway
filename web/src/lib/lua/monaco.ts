import type * as Monaco from "monaco-editor";

import {
  LUA_KEYWORDS,
  functionDetail,
  functionDocs,
  lookupFunction,
  memberSnippet,
  type LuaApiIndex,
  type LuaFunction,
} from "./api";

const MEMBER = /([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z0-9_]*)$/;

type Call = { namespace: string; name: string; argument: number };

export function registerLuaApi(
  monaco: typeof Monaco,
  index: LuaApiIndex,
): Monaco.IDisposable {
  const disposables = [
    monaco.languages.registerCompletionItemProvider("lua", {
      triggerCharacters: ["."],
      provideCompletionItems: (model, position) =>
        completions(monaco, index, model, position),
    }),
    monaco.languages.registerHoverProvider("lua", {
      provideHover: (model, position) => hover(index, model, position),
    }),
    monaco.languages.registerSignatureHelpProvider("lua", {
      signatureHelpTriggerCharacters: ["(", ","],
      provideSignatureHelp: (model, position) =>
        signatureHelp(index, model, position),
    }),
  ];

  return {
    dispose: () => disposables.forEach((disposable) => disposable.dispose()),
  };
}

function textBefore(
  model: Monaco.editor.ITextModel,
  position: Monaco.Position,
): string {
  return model.getValueInRange({
    startLineNumber: position.lineNumber,
    startColumn: 1,
    endLineNumber: position.lineNumber,
    endColumn: position.column,
  });
}

function completions(
  monaco: typeof Monaco,
  index: LuaApiIndex,
  model: Monaco.editor.ITextModel,
  position: Monaco.Position,
): Monaco.languages.CompletionList {
  const word = model.getWordUntilPosition(position);
  const range: Monaco.IRange = {
    startLineNumber: position.lineNumber,
    endLineNumber: position.lineNumber,
    startColumn: word.startColumn,
    endColumn: word.endColumn,
  };

  const member = MEMBER.exec(textBefore(model, position));
  const namespace = member ? index.namespaces.get(member[1]) : undefined;

  if (namespace) {
    const functions = namespace.functions.map((fn) => ({
      label: fn.name,
      kind: monaco.languages.CompletionItemKind.Function,
      detail: functionDetail(fn),
      documentation: { value: functionDocs(index, fn).join("\n\n") },
      insertText: memberSnippet(fn),
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      range,
    }));

    const fields = namespace.fields.map((field) => ({
      label: field.name,
      kind: monaco.languages.CompletionItemKind.Field,
      detail: field.type,
      insertText: field.name,
      range,
    }));

    return { suggestions: [...functions, ...fields] };
  }

  if (member) {
    return { suggestions: [] };
  }

  const namespaces = [...index.namespaces.values()].map((entry) => ({
    label: entry.name,
    kind: monaco.languages.CompletionItemKind.Module,
    detail: `${entry.functions.length} functions`,
    insertText: entry.name,
    range,
  }));

  const globals = [...index.globals.values()].map((global) => ({
    label: global.name,
    kind: monaco.languages.CompletionItemKind.Variable,
    detail: global.type,
    insertText: global.name,
    range,
  }));

  const keywords = LUA_KEYWORDS.map((keyword) => ({
    label: keyword,
    kind: monaco.languages.CompletionItemKind.Keyword,
    insertText: keyword,
    range,
  }));

  return { suggestions: [...namespaces, ...globals, ...keywords] };
}

function hover(
  index: LuaApiIndex,
  model: Monaco.editor.ITextModel,
  position: Monaco.Position,
): Monaco.languages.Hover | null {
  const word = model.getWordAtPosition(position);

  if (!word) {
    return null;
  }

  const range = {
    startLineNumber: position.lineNumber,
    endLineNumber: position.lineNumber,
    startColumn: word.startColumn,
    endColumn: word.endColumn,
  };

  const prefix = MEMBER.exec(
    textBefore(model, position.with(undefined, word.startColumn)),
  );

  if (!prefix) {
    const namespace = index.namespaces.get(word.word);
    const global = index.globals.get(word.word);

    if (namespace) {
      return {
        range,
        contents: [
          { value: `\`\`\`lua\n${namespace.name}\n\`\`\`` },
          {
            value: namespace.functions
              .map((fn) => `- \`${fn.name}\``)
              .join("\n"),
          },
        ],
      };
    }

    return global
      ? { range, contents: [{ value: `\`\`\`lua\n${global.declaration}\n\`\`\`` }] }
      : null;
  }

  const fn = lookupFunction(index, prefix[1], word.word);

  if (fn) {
    return {
      range,
      contents: functionDocs(index, fn).map((value) => ({ value })),
    };
  }

  const field = index.namespaces
    .get(prefix[1])
    ?.fields.find((entry) => entry.name === word.word);

  return field
    ? { range, contents: [{ value: `\`\`\`lua\n${field.declaration}\n\`\`\`` }] }
    : null;
}

function signatureHelp(
  index: LuaApiIndex,
  model: Monaco.editor.ITextModel,
  position: Monaco.Position,
): Monaco.languages.SignatureHelpResult | null {
  const call = enclosingCall(textBefore(model, position));
  const fn = call && lookupFunction(index, call.namespace, call.name);

  if (!call || !fn) {
    return null;
  }

  return {
    value: {
      signatures: [signature(index, fn)],
      activeSignature: 0,
      activeParameter: Math.min(call.argument, Math.max(fn.params.length - 1, 0)),
    },
    dispose: () => {},
  };
}

function signature(
  index: LuaApiIndex,
  fn: LuaFunction,
): Monaco.languages.SignatureInformation {
  return {
    label: functionDetail(fn),
    documentation: { value: functionDocs(index, fn).slice(1).join("\n\n") },
    parameters: fn.params.map((param) => ({
      label: param.declaration,
      documentation: param.type,
    })),
  };
}

function enclosingCall(text: string): Call | null {
  let depth = 0;
  let argument = 0;

  for (let cursor = text.length - 1; cursor >= 0; cursor -= 1) {
    const character = text[cursor];

    if (character === ")") {
      depth += 1;
    } else if (character === ",") {
      if (depth === 0) {
        argument += 1;
      }
    } else if (character === "(") {
      if (depth > 0) {
        depth -= 1;
        continue;
      }

      const target = MEMBER.exec(text.slice(0, cursor));

      return target
        ? { namespace: target[1], name: target[2], argument }
        : null;
    }
  }

  return null;
}
