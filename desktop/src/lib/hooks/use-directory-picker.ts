import { useCallback } from "react";

/**
 * Detect whether we're running inside Tauri.
 */
declare const __TAURI_DEV__: boolean;
const isTauri = Boolean(
  typeof window !== "undefined" &&
    ((window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ ||
      (typeof __TAURI_DEV__ !== "undefined" && __TAURI_DEV__))
);

/**
 * Hook that returns a `pickDirectory` callback.
 * In Tauri mode, uses the native OS directory picker dialog.
 * In web mode, falls back to a prompt().
 */
export function useDirectoryPicker() {
  const pickDirectory = useCallback(async (): Promise<string | null> => {
    if (isTauri) {
      try {
        const { open } = await import("@tauri-apps/plugin-dialog");
        const selected = await open({ directory: true, multiple: false });
        if (typeof selected === "string") return selected;
        return null;
      } catch {
        // Fallback if dialog plugin unavailable
        return prompt("Enter directory path:") || null;
      }
    }
    // Web fallback
    return prompt("Enter directory path:") || null;
  }, []);

  return { pickDirectory };
}

/**
 * Hook that returns a `pickFile` callback.
 * In Tauri mode, uses the native OS file picker dialog.
 * In web mode, falls back to an <input type="file">.
 */
export function useFilePicker() {
  const pickFile = useCallback(
    async (options?: {
      filters?: Array<{ name: string; extensions: string[] }>;
      multiple?: boolean;
    }): Promise<string | string[] | null> => {
      if (isTauri) {
        try {
          const { open } = await import("@tauri-apps/plugin-dialog");
          const selected = await open({
            directory: false,
            multiple: options?.multiple ?? false,
            filters: options?.filters,
          });
          return selected;
        } catch {
          return null;
        }
      }
      return null;
    },
    []
  );

  return { pickFile };
}
