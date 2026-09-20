export type LuaField = {
  readonly name: string;
  readonly type: string;
  readonly optional: boolean;
  readonly declaration: string;
};

export type LuaFunction = {
  readonly name: string;
  readonly params: readonly LuaField[];
  readonly returns: string | null | undefined;
  readonly scope: string | null | undefined;
  readonly signature: string;
};

export type LuaNamespace = {
  readonly name: string;
  readonly fields: readonly LuaField[];
  readonly functions: readonly LuaFunction[];
};

export type LuaClass = {
  readonly name: string;
  readonly fields: readonly LuaField[];
};

export type LuaAlias = {
  readonly name: string;
  readonly type: string;
};

export type LuaApi = {
  readonly namespaces: readonly LuaNamespace[];
  readonly globals: readonly LuaField[];
  readonly classes: readonly LuaClass[];
  readonly aliases: readonly LuaAlias[];
  readonly definitions: string;
};

export type LuaApiIndex = {
  readonly api: LuaApi;
  readonly namespaces: ReadonlyMap<string, LuaNamespace>;
  readonly globals: ReadonlyMap<string, LuaField>;
  readonly types: ReadonlyMap<string, string>;
};

const KEYWORDS = [
  "and",
  "break",
  "do",
  "else",
  "elseif",
  "end",
  "false",
  "for",
  "function",
  "if",
  "in",
  "local",
  "nil",
  "not",
  "or",
  "repeat",
  "return",
  "then",
  "true",
  "until",
  "while",
];

export const LUA_KEYWORDS: readonly string[] = KEYWORDS;

export function indexLuaApi(api: LuaApi): LuaApiIndex {
  const namespaces = new Map(
    api.namespaces.map((namespace) => [namespace.name, namespace]),
  );

  const globals = new Map(api.globals.map((global) => [global.name, global]));

  const types = new Map<string, string>();

  for (const alias of api.aliases) {
    types.set(`gw.${alias.name}`, alias.type);
  }

  for (const cls of api.classes) {
    types.set(
      `gw.${cls.name}`,
      `{ ${cls.fields.map((field) => field.declaration).join(", ")} }`,
    );
  }

  return { api, namespaces, globals, types };
}

export function lookupFunction(
  index: LuaApiIndex,
  namespace: string,
  name: string,
): LuaFunction | undefined {
  return index.namespaces
    .get(namespace)
    ?.functions.find((fn) => fn.name === name);
}

export function functionDetail(fn: LuaFunction): string {
  return fn.returns ? `${fn.signature}: ${fn.returns}` : fn.signature;
}

export function functionDocs(
  index: LuaApiIndex,
  fn: LuaFunction,
): readonly string[] {
  const lines = [`\`\`\`lua\n${functionDetail(fn)}\n\`\`\``];

  if (fn.scope) {
    lines.push(`Requires the \`${fn.scope}\` scope.`);
  }

  const referenced = [...fn.params.map((param) => param.type), fn.returns ?? ""]
    .flatMap((type) => [...type.matchAll(/gw\.[A-Za-z0-9_]+/g)])
    .map((match) => match[0]);

  const expanded = [...new Set(referenced)]
    .map((name) => {
      const type = index.types.get(name);
      return type ? `\`${name}\` = \`${type}\`` : null;
    })
    .filter((line): line is string => line !== null);

  return [...lines, ...expanded];
}

export function memberSnippet(fn: LuaFunction): string {
  const params = fn.params
    .map((param, position) => `\${${position + 1}:${param.name}}`)
    .join(", ");

  return `${fn.name}(${params})`;
}

export function callText(namespace: string, fn: LuaFunction): string {
  const params = fn.params.map((param) => param.name).join(", ");

  return `${namespace}.${fn.name}(${params})`;
}
