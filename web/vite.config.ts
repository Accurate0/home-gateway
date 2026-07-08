import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import relay from "vite-plugin-relay";
import path from "path";

// In dev, proxy the API + subscription websocket to a locally running backend
// (cargo run serves on :8000) so the SPA can use same-origin relative URLs.
const BACKEND = process.env.BACKEND_URL ?? "http://localhost:8000";

// https://vite.dev/config/
export default defineConfig({
  plugins: [relay, react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    proxy: {
      "/v1": {
        target: BACKEND,
        changeOrigin: true,
        ws: true,
      },
    },
  },
});
