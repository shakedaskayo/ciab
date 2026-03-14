import { useState, useEffect } from "react";
import { useConnectionStore } from "@/lib/stores/connection-store";
import { health } from "@/lib/api/endpoints";
import { Circle, Check } from "lucide-react";
import { toast } from "sonner";
import CiabLogo from "@/components/shared/CiabLogo";

export default function Settings() {
  const serverUrl = useConnectionStore((s) => s.serverUrl);
  const apiKey = useConnectionStore((s) => s.apiKey);
  const connected = useConnectionStore((s) => s.connected);
  const setServerUrl = useConnectionStore((s) => s.setServerUrl);
  const setApiKey = useConnectionStore((s) => s.setApiKey);
  const setConnected = useConnectionStore((s) => s.setConnected);

  const [urlInput, setUrlInput] = useState(serverUrl);
  const [keyInput, setKeyInput] = useState(apiKey);
  const [testing, setTesting] = useState(false);

  useEffect(() => {
    setUrlInput(serverUrl);
    setKeyInput(apiKey);
  }, [serverUrl, apiKey]);

  const testConnection = async () => {
    setTesting(true);
    try {
      setServerUrl(urlInput);
      setApiKey(keyInput);
      await health.check();
      setConnected(true);
      toast.success("Connected to CIAB server");
    } catch {
      setConnected(false);
      toast.error("Failed to connect to server");
    } finally {
      setTesting(false);
    }
  };

  const handleSave = () => {
    setServerUrl(urlInput);
    setApiKey(keyInput);
    toast.success("Settings saved");
  };

  return (
    <div className="max-w-lg space-y-5 animate-fade-in">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Settings</h1>
        <p className="text-sm text-ciab-text-muted mt-0.5">
          Configure your CIAB connection
        </p>
      </div>

      {/* Server Connection */}
      <div className="card p-4 space-y-4">
        <div className="flex items-center justify-between">
          <span className="label mb-0">Server Connection</span>
          <div className="flex items-center gap-1.5">
            <Circle
              className={`w-2 h-2 ${
                connected
                  ? "text-state-running fill-state-running"
                  : "text-state-failed fill-state-failed"
              }`}
            />
            <span className={`text-[10px] font-mono ${connected ? "text-state-running" : "text-state-failed"}`}>
              {connected ? "CONNECTED" : "OFFLINE"}
            </span>
          </div>
        </div>

        <div>
          <label className="label">Server URL</label>
          <input
            type="text"
            value={urlInput}
            onChange={(e) => setUrlInput(e.target.value)}
            placeholder="http://localhost:8080"
            className="input w-full font-mono text-xs"
          />
        </div>

        <div>
          <label className="label">
            API Key <span className="text-ciab-text-muted/50 normal-case tracking-normal">(optional)</span>
          </label>
          <input
            type="password"
            value={keyInput}
            onChange={(e) => setKeyInput(e.target.value)}
            placeholder="Enter API key..."
            className="input w-full font-mono text-xs"
          />
        </div>

        <div className="flex items-center gap-2 pt-1">
          <button
            onClick={testConnection}
            disabled={testing}
            className="btn-secondary text-xs px-3 py-1.5"
          >
            {testing ? "Testing..." : "Test Connection"}
          </button>
          <button
            onClick={handleSave}
            className="btn-primary flex items-center gap-1.5 text-xs px-3 py-1.5"
          >
            <Check className="w-3.5 h-3.5" />
            Save
          </button>
        </div>
      </div>

      {/* About */}
      <div className="card p-4">
        <span className="label">About</span>
        <div className="flex items-center gap-3 mt-2">
          <CiabLogo size={36} />
          <div>
            <p className="text-sm font-medium">
              CIAB <span className="text-ciab-text-muted font-mono text-xs">v0.1.0</span>
            </p>
            <p className="text-xs text-ciab-text-muted mt-0.5">
              Manage coding agent sandboxes. Supports Claude Code, Codex, Gemini CLI, and Cursor.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
