import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

/**
 * Detect whether we're running inside Tauri (desktop app) vs a plain browser.
 * When served by the CIAB server itself (web mode), API calls should target
 * the same origin so they work regardless of hostname/tunnel URL.
 *
 * We use a build-time flag (__TAURI_DEV__) set by Vite when `tauri dev` is
 * running, plus the runtime check for __TAURI_INTERNALS__. The build-time
 * flag is critical because __TAURI_INTERNALS__ may not be injected yet when
 * this module first evaluates, which would cause the default URL to be
 * window.location.origin (the Vite dev server) instead of localhost:9090.
 * The Vite dev proxy does not reliably forward SSE streams.
 */
declare const __TAURI_DEV__: boolean;
const isTauri = Boolean(
  typeof window !== "undefined" &&
    ((window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ ||
      (typeof __TAURI_DEV__ !== "undefined" && __TAURI_DEV__))
);

const DEFAULT_SERVER_URL = isTauri
  ? "http://localhost:9090"
  : window.location.origin;

interface ConnectionState {
  serverUrl: string;
  apiKey: string;
  connected: boolean;
  lastError: string | null;
  setServerUrl: (url: string) => void;
  setApiKey: (key: string) => void;
  setConnected: (connected: boolean) => void;
  setLastError: (error: string | null) => void;
}

export function getServerUrl(): string {
  return useConnectionStore.getState().serverUrl;
}

export function getApiKey(): string {
  return useConnectionStore.getState().apiKey;
}

export const useConnectionStore = create<ConnectionState>()(
  persist(
    (set) => ({
      serverUrl: DEFAULT_SERVER_URL,
      apiKey: "",
      connected: false,
      lastError: null,
      setServerUrl: (url) => set({ serverUrl: url }),
      setApiKey: (key) => set({ apiKey: key }),
      setConnected: (connected) => set({ connected }),
      setLastError: (error) => set({ lastError: error }),
    }),
    {
      name: "ciab-connection",
      version: 3,
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        serverUrl: state.serverUrl,
        apiKey: state.apiKey,
      }),
      migrate: (persisted: unknown, version: number) => {
        const state = persisted as { serverUrl?: string; apiKey?: string };
        if (version < 2) {
          if (!isTauri) {
            state.serverUrl = window.location.origin;
          } else if (state.serverUrl === "http://localhost:8080") {
            state.serverUrl = "http://localhost:9090";
          }
        }
        if (version < 3) {
          // v3: In Tauri mode, fix persisted URLs that point at the Vite dev
          // server (localhost:5199) — the Vite proxy doesn't support SSE.
          // Always reset to the direct backend URL.
          if (isTauri && state.serverUrl?.includes("localhost:5199")) {
            state.serverUrl = "http://localhost:9090";
          }
        }
        return state as ConnectionState;
      },
    }
  )
);
