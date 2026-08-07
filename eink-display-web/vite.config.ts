import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { viteSingleFile } from "vite-plugin-singlefile";
import tailwindcss from "@tailwindcss/vite";
import relay from "vite-plugin-relay";
import path from "path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    relay,
    react(),
    viteSingleFile(),
    tailwindcss(),
  ],
  build: {
    assetsInlineLimit: Number.MAX_SAFE_INTEGER,
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
