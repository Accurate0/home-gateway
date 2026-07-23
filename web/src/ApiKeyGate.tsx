import { useState } from "react";
import { setApiKey } from "./auth";

export default function ApiKeyGate() {
  const [value, setValue] = useState("");

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!value.trim()) return;
    setApiKey(value);
    location.reload();
  };

  return (
    <div className="mx-auto grid min-h-screen max-w-md place-items-center px-6 text-center">
      <form onSubmit={submit} className="w-full">
        <h1 className="mb-2 text-lg font-semibold">API key required</h1>
        <p className="text-muted-foreground mb-4 text-sm">
          Enter an API key to access the gateway.
        </p>
        <input
          type="password"
          autoFocus
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="X-Api-Key"
          className="border-input mb-4 w-full rounded-md border px-3 py-2 text-sm"
        />
        <button
          type="submit"
          className="bg-primary text-primary-foreground w-full rounded-md px-3 py-2 text-sm font-medium"
        >
          Continue
        </button>
      </form>
    </div>
  );
}
