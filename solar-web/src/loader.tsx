import type { SolarCurrentResponse, SolarHistoryResponse } from "./types";

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

export const loader = async () => {
  const historyPromise = fetch(`${API_BASE_URL}/solar/history`)
    .then((r) => r.json())
    .then((json) => json as SolarHistoryResponse);

  const currentPromise = fetch(`${API_BASE_URL}/solar/current`)
    .then((r) => r.json())
    .then((json) => json as SolarCurrentResponse);

  const [historyResponse, currentResponse] = await Promise.all([
    historyPromise,
    currentPromise,
  ]);

  return { ...historyResponse, current: currentResponse };
};
