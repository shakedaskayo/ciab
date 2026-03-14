import { useEffect } from "react";
import { useConnectionStore } from "@/lib/stores/connection-store";
import { useUIStore } from "@/lib/stores/ui-store";
import { health } from "@/lib/api/endpoints";
import { Menu } from "lucide-react";

export default function TopBar() {
  const connected = useConnectionStore((s) => s.connected);
  const serverUrl = useConnectionStore((s) => s.serverUrl);
  const setConnected = useConnectionStore((s) => s.setConnected);
  const setLastError = useConnectionStore((s) => s.setLastError);
  const setSidebarOpen = useUIStore((s) => s.setSidebarOpen);

  useEffect(() => {
    const check = async () => {
      try {
        await health.check();
        setConnected(true);
        setLastError(null);
      } catch {
        setConnected(false);
        setLastError("Cannot reach server");
      }
    };

    check();
    const interval = setInterval(check, 15000);
    return () => clearInterval(interval);
  }, [serverUrl, setConnected, setLastError]);

  const displayUrl = serverUrl.replace(/^https?:\/\//, "");

  return (
    <header className="h-10 border-b border-ciab-border bg-ciab-bg-secondary/50 backdrop-blur-md flex items-center justify-between px-4 sticky top-0 z-10">
      {/* Hamburger — mobile only */}
      <button
        onClick={() => setSidebarOpen(true)}
        className="md:hidden p-1.5 -ml-1.5 rounded-md text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
        aria-label="Open menu"
      >
        <Menu className="w-5 h-5" />
      </button>

      {/* Spacer for desktop (no hamburger) */}
      <div className="hidden md:block" />

      <div className="flex items-center gap-2">
        <span className="text-[10px] text-ciab-text-muted font-mono tracking-wide hidden sm:inline">
          {displayUrl}
        </span>
        <div
          className={`flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-mono font-medium tracking-wider ${
            connected
              ? "bg-state-running/10 text-state-running"
              : "bg-state-failed/10 text-state-failed"
          }`}
        >
          <span
            className={`w-1.5 h-1.5 rounded-full ${
              connected
                ? "bg-state-running animate-pulse-slow"
                : "bg-state-failed"
            }`}
          />
          {connected ? "LIVE" : "OFFLINE"}
        </div>
      </div>
    </header>
  );
}
