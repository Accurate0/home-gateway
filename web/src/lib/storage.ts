export function readStored<T>(key: string, fallback: T): T {
  try {
    const raw = window.localStorage.getItem(key);

    return raw === null ? fallback : (JSON.parse(raw) as T);
  } catch {
    return fallback;
  }
}

export function writeStored<T>(key: string, value: T): void {
  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch {
    return;
  }
}

export function removeStored(key: string): void {
  try {
    window.localStorage.removeItem(key);
  } catch {
    return;
  }
}
