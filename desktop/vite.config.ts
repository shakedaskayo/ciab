import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import http from "node:http";

const host = process.env.TAURI_DEV_HOST;

// Detect Tauri dev mode at build time so the frontend can reliably
// distinguish between running inside the Tauri webview vs a plain browser.
const isTauriDev = Boolean(process.env.TAURI_ENV_PLATFORM);

const BACKEND_URL = "http://localhost:9090";

/**
 * Vite plugin that intercepts SSE requests (/api/…/stream) and pipes them
 * directly from the backend, bypassing http-proxy which buffers chunked
 * responses and breaks Server-Sent Events.
 */
function sseProxyPlugin() {
  return {
    name: "sse-proxy",
    configureServer(server: { middlewares: { use: (fn: unknown) => void } }) {
      server.middlewares.use((req: http.IncomingMessage, res: http.ServerResponse, next: () => void) => {
        if (!req.url || !req.url.includes("/stream")) {
          return next();
        }
        // Only handle SSE-looking requests under /api
        if (!req.url.startsWith("/api")) {
          return next();
        }

        const targetUrl = new URL(req.url, BACKEND_URL);
        const proxyReq = http.request(
          targetUrl,
          {
            method: req.method,
            headers: {
              ...req.headers,
              host: new URL(BACKEND_URL).host,
            },
          },
          (proxyRes) => {
            res.writeHead(proxyRes.statusCode ?? 200, {
              "Content-Type": "text/event-stream",
              "Cache-Control": "no-cache, no-transform",
              Connection: "keep-alive",
              "X-Accel-Buffering": "no",
              "Access-Control-Allow-Origin": "*",
            });
            // Pipe chunks directly without buffering
            proxyRes.on("data", (chunk: Buffer) => {
              res.write(chunk);
            });
            proxyRes.on("end", () => {
              res.end();
            });
          }
        );

        proxyReq.on("error", () => {
          if (!res.headersSent) {
            res.writeHead(502);
            res.end("Backend unavailable");
          }
        });

        req.pipe(proxyReq);
      });
    },
  };
}

export default defineConfig(async () => ({
  plugins: [react(), sseProxyPlugin()],
  define: {
    __TAURI_DEV__: JSON.stringify(isTauriDev),
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    port: 5199,
    strictPort: false,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 5174,
        }
      : undefined,
    proxy: {
      "/api": {
        target: BACKEND_URL,
        changeOrigin: true,
      },
      "/health": {
        target: BACKEND_URL,
        changeOrigin: true,
      },
      "/ws": {
        target: "ws://localhost:9090",
        ws: true,
      },
    },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
