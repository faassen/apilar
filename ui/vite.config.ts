import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";

export default defineConfig({
  plugins: [solidPlugin()],
  server: {
    port: 3000,
    proxy: {
      "/ws": {
        target: "ws://localhost:4000/",
        ws: true,
      },
    },
  },
  build: {
    target: "esnext",
  },
});
