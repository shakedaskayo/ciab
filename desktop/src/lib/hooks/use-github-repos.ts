import { useState, useCallback, useRef } from "react";

declare const __TAURI_DEV__: boolean;
const isTauri = Boolean(
  typeof window !== "undefined" &&
    ((window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ ||
      (typeof __TAURI_DEV__ !== "undefined" && __TAURI_DEV__))
);

export interface GitHubRepo {
  fullName: string;
  url: string;
  description: string;
  isPrivate: boolean;
  defaultBranch: string;
}

/**
 * Hook to search/list GitHub repos using the `gh` CLI via Tauri shell plugin.
 * Falls back gracefully when not in Tauri or `gh` is not installed.
 */
export function useGitHubRepos() {
  const [repos, setRepos] = useState<GitHubRepo[]>([]);
  const [loading, setLoading] = useState(false);
  const [ghAvailable, setGhAvailable] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef(false);

  const runGhCommand = useCallback(
    async (args: string[]): Promise<string | null> => {
      if (!isTauri) return null;
      try {
        const { Command } = await import("@tauri-apps/plugin-shell");
        const result = await Command.create("gh", args).execute();
        if (result.code === 0) return result.stdout;
        return null;
      } catch {
        return null;
      }
    },
    []
  );

  const checkAvailability = useCallback(async () => {
    if (!isTauri) {
      setGhAvailable(false);
      return false;
    }
    const result = await runGhCommand(["auth", "status"]);
    const available = result !== null;
    setGhAvailable(available);
    return available;
  }, [runGhCommand]);

  const searchRepos = useCallback(
    async (query: string) => {
      setLoading(true);
      setError(null);
      abortRef.current = false;

      try {
        // gh search repos <query> --json fullName,url,description,isPrivate,defaultBranch --limit 20
        const args = query.trim()
          ? [
              "search",
              "repos",
              query,
              "--json",
              "fullName,url,description,isPrivate,defaultBranch",
              "--limit",
              "20",
            ]
          : [
              "repo",
              "list",
              "--json",
              "nameWithOwner,url,description,isPrivate,defaultBranchRef",
              "--limit",
              "20",
            ];

        const output = await runGhCommand(args);
        if (abortRef.current) return;

        if (!output) {
          setError("Failed to fetch repos from GitHub CLI");
          setRepos([]);
          return;
        }

        const parsed = JSON.parse(output);
        const mapped: GitHubRepo[] = parsed.map(
          (r: Record<string, unknown>) => ({
            fullName: r.fullName || r.nameWithOwner || "",
            url: (r.url as string) || "",
            description: (r.description as string) || "",
            isPrivate: Boolean(r.isPrivate),
            defaultBranch:
              (r.defaultBranch as string) ||
              ((r.defaultBranchRef as Record<string, string>)?.name ?? "main"),
          })
        );
        setRepos(mapped);
      } catch (e) {
        if (!abortRef.current) {
          setError(e instanceof Error ? e.message : "Search failed");
          setRepos([]);
        }
      } finally {
        if (!abortRef.current) setLoading(false);
      }
    },
    [runGhCommand]
  );

  const cancelSearch = useCallback(() => {
    abortRef.current = true;
    setLoading(false);
  }, []);

  return {
    repos,
    loading,
    ghAvailable,
    error,
    checkAvailability,
    searchRepos,
    cancelSearch,
  };
}
