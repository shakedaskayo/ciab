import { useState } from "react";
import { useSandboxes, useCreateSandbox } from "@/lib/hooks/use-sandboxes";
import SandboxGrid from "@/features/dashboard/SandboxGrid";
import QuickStats from "@/features/dashboard/QuickStats";
import LoadingSpinner from "@/components/shared/LoadingSpinner";
import EmptyState from "@/components/shared/EmptyState";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";
import CreateSandboxDialog from "@/features/sandbox/CreateSandboxDialog";
import { Container, Plus, Zap, RefreshCw } from "lucide-react";
import { useNavigate } from "react-router";
import { useQueryClient } from "@tanstack/react-query";

const QUICK_LAUNCH = [
  { provider: "claude-code", label: "Claude Code", desc: "Anthropic" },
  { provider: "codex", label: "Codex", desc: "OpenAI" },
  { provider: "gemini", label: "Gemini CLI", desc: "Google" },
  { provider: "cursor", label: "Cursor", desc: "Anysphere" },
] as const;

export default function Dashboard() {
  const { data: sandboxList, isLoading, isFetching } = useSandboxes();
  const createSandbox = useCreateSandbox();
  const navigate = useNavigate();
  const qc = useQueryClient();
  const [showDialog, setShowDialog] = useState(false);
  const [spinning, setSpinning] = useState(false);

  const handleRefresh = () => {
    setSpinning(true);
    qc.invalidateQueries({ queryKey: ["sandboxes"] });
    setTimeout(() => setSpinning(false), 600);
  };

  const handleQuickLaunch = (provider: string) => {
    createSandbox.mutate(
      { agent_provider: provider },
      {
        onSuccess: () => {
          // Sandbox is provisioning asynchronously — navigate to sandboxes list
          navigate("/sandboxes");
        },
      }
    );
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  const running = sandboxList?.filter((s) => s.state === "running") ?? [];
  const provisioning =
    sandboxList?.filter(
      (s) => s.state === "creating" || s.state === "pending"
    ) ?? [];

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">Dashboard</h1>
          <p className="text-sm text-ciab-text-muted mt-0.5">
            Manage your coding agent sandboxes
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleRefresh}
            className="p-2 rounded-md text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
            title="Refresh"
          >
            <RefreshCw
              className={`w-4 h-4 ${spinning || isFetching ? "animate-spin" : ""}`}
            />
          </button>
          <button
            onClick={() => setShowDialog(true)}
            className="btn-primary flex items-center gap-2"
          >
            <Plus className="w-4 h-4" />
            New Sandbox
          </button>
        </div>
      </div>

      {/* Quick launch */}
      <div>
        <div className="flex items-center gap-2 mb-2.5">
          <Zap className="w-3.5 h-3.5 text-ciab-copper" />
          <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
            Quick Launch
          </span>
        </div>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
          {QUICK_LAUNCH.map((item) => (
            <button
              key={item.provider}
              onClick={() => handleQuickLaunch(item.provider)}
              disabled={createSandbox.isPending}
              className="card-hover group p-3 text-left disabled:opacity-50"
            >
              <div className="flex items-center gap-2.5">
                <div className="w-9 h-9 rounded-md bg-ciab-bg-hover flex items-center justify-center group-hover:scale-105 transition-transform">
                  <AgentProviderIcon provider={item.provider} size={20} />
                </div>
                <div>
                  <p className="text-sm font-medium text-ciab-text-primary">
                    {item.label}
                  </p>
                  <p className="text-[10px] font-mono text-ciab-text-muted">
                    {item.desc}
                  </p>
                </div>
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Stats */}
      <QuickStats sandboxes={sandboxList ?? []} />

      {/* Provisioning sandboxes */}
      {provisioning.length > 0 && (
        <div>
          <div className="flex items-center gap-2 mb-2.5">
            <div className="w-1.5 h-1.5 rounded-full bg-state-creating animate-pulse" />
            <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
              Provisioning ({provisioning.length})
            </span>
          </div>
          <SandboxGrid sandboxes={provisioning} />
        </div>
      )}

      {/* Active sandboxes */}
      {running.length > 0 && (
        <div>
          <div className="flex items-center gap-2 mb-2.5">
            <div className="w-1.5 h-1.5 rounded-full bg-state-running animate-pulse-slow" />
            <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
              Active ({running.length})
            </span>
          </div>
          <SandboxGrid sandboxes={running} />
        </div>
      )}

      {/* All sandboxes */}
      {sandboxList && sandboxList.length > 0 ? (
        <div>
          <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider mb-2.5 block">
            All Sandboxes ({sandboxList.length})
          </span>
          <SandboxGrid sandboxes={sandboxList} />
        </div>
      ) : (
        <EmptyState
          icon={Container}
          title="No sandboxes yet"
          description="Launch your first coding agent sandbox using the quick launch buttons above."
          action={
            <button onClick={() => setShowDialog(true)} className="btn-primary">
              Create Sandbox
            </button>
          }
        />
      )}

      {showDialog && (
        <CreateSandboxDialog
          onClose={() => setShowDialog(false)}
          onSuccess={() => {
            setShowDialog(false);
            navigate("/sandboxes");
          }}
        />
      )}
    </div>
  );
}
