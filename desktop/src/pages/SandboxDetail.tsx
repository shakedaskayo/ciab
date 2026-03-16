import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router";
import {
  useSandbox,
  useSandboxAction,
  useDeleteSandbox,
} from "@/lib/hooks/use-sandboxes";
import SandboxStateBadge from "@/components/shared/SandboxStateBadge";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";
import LoadingSpinner from "@/components/shared/LoadingSpinner";
import ChatView from "@/features/chat/ChatView";
import TerminalView from "@/features/terminal/TerminalView";
import FileBrowser from "@/features/files/FileBrowser";
import LogViewer from "@/features/logs/LogViewer";
import StatsView from "@/features/stats/StatsView";
import { getAvailableActions } from "@/lib/utils/sandbox-state";
import { truncateId } from "@/lib/utils/format";
import {
  useProvisioningProgress,
  PROVISIONING_STEPS,
} from "@/lib/hooks/use-provisioning";
import {
  ArrowLeft,
  MessageSquare,
  Terminal,
  FolderTree,
  ScrollText,
  BarChart3,
  Play,
  Square,
  Pause,
  RotateCcw,
  Trash2,
} from "lucide-react";

type Tab = "chat" | "terminal" | "files" | "logs" | "stats";

const tabs: Array<{ id: Tab; label: string; icon: typeof MessageSquare }> = [
  { id: "chat", label: "Chat", icon: MessageSquare },
  { id: "terminal", label: "Terminal", icon: Terminal },
  { id: "files", label: "Files", icon: FolderTree },
  { id: "logs", label: "Logs", icon: ScrollText },
  { id: "stats", label: "Stats", icon: BarChart3 },
];

const actionConfig = {
  start: { icon: Play, label: "Start", style: "btn-secondary" },
  stop: { icon: Square, label: "Stop", style: "btn-secondary" },
  pause: { icon: Pause, label: "Pause", style: "btn-secondary" },
  resume: { icon: RotateCcw, label: "Resume", style: "btn-secondary" },
  delete: { icon: Trash2, label: "Delete", style: "btn-danger" },
};

export default function SandboxDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: sandbox, isLoading } = useSandbox(id!);
  const sandboxAction = useSandboxAction();
  const deleteSandbox = useDeleteSandbox();
  const [activeTab, setActiveTab] = useState<Tab>("chat");
  const [initialTabSet, setInitialTabSet] = useState(false);

  const isProvisioning =
    sandbox?.state === "creating" || sandbox?.state === "pending";

  const progress = useProvisioningProgress(id!, isProvisioning);

  // Auto-switch to logs tab when sandbox is provisioning (only on first load)
  useEffect(() => {
    if (sandbox && !initialTabSet) {
      setInitialTabSet(true);
      if (
        sandbox.state === "creating" ||
        sandbox.state === "pending"
      ) {
        setActiveTab("logs");
      }
    }
  }, [sandbox, initialTabSet]);

  if (isLoading || !sandbox) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  const actions = getAvailableActions(sandbox.state);

  return (
    <div className="flex flex-col h-[calc(100vh-7rem)] animate-fade-in">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 mb-3">
        <div className="flex items-center gap-2.5 min-w-0">
          <button
            onClick={() => navigate(-1)}
            className="p-1 text-ciab-text-muted hover:text-ciab-text-primary transition-colors flex-shrink-0"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div className="w-8 h-8 rounded-md bg-ciab-bg-elevated flex items-center justify-center flex-shrink-0">
            <AgentProviderIcon provider={sandbox.agent_provider} size={18} />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <h1 className="text-base font-semibold truncate">
                {sandbox.name ?? truncateId(sandbox.id)}
              </h1>
              <SandboxStateBadge state={sandbox.state} />
            </div>
            <p className="text-[10px] text-ciab-text-muted font-mono truncate">
              {sandbox.id}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-1.5 flex-shrink-0 pl-7 sm:pl-0">
          {actions.map((action) => {
            const config = actionConfig[action];
            return (
              <button
                key={action}
                onClick={() =>
                  action === "delete"
                    ? deleteSandbox.mutate(sandbox.id, {
                        onSuccess: () => navigate("/sandboxes"),
                      })
                    : sandboxAction.mutate({ id: sandbox.id, action })
                }
                className={`${config.style} flex items-center gap-1.5 text-xs px-2.5 py-1.5`}
              >
                <config.icon className="w-3.5 h-3.5" />
                <span className="hidden sm:inline">{config.label}</span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Provisioning progress bar — shown in header area when provisioning */}
      {isProvisioning && (
        <div className="mb-3 px-1 space-y-1.5 animate-fade-in">
          {/* Progress bar */}
          <div className="h-1.5 bg-ciab-bg-hover rounded-full overflow-hidden">
            <div
              className="h-full rounded-full transition-all duration-700 ease-out bg-gradient-to-r from-ciab-copper/80 to-ciab-copper"
              style={{ width: `${Math.max(progress.percent, 3)}%` }}
            />
          </div>

          {/* Step info */}
          <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-1.5 min-w-0">
              <div className="w-1.5 h-1.5 rounded-full bg-ciab-copper animate-pulse" />
              <span className="text-[11px] font-mono text-ciab-text-secondary truncate">
                {progress.currentStepLabel ?? "Initializing..."}
              </span>
            </div>
            <span className="text-[11px] font-mono text-ciab-text-muted flex-shrink-0">
              {progress.stepIndex >= 0
                ? `${progress.stepIndex + 1}/${progress.totalSteps}`
                : ""}
            </span>
          </div>

          {/* Step segments */}
          <div className="flex items-center gap-0.5">
            {PROVISIONING_STEPS.map((step, i) => (
              <div
                key={step.id}
                className={`h-[3px] flex-1 rounded-full transition-all duration-500 ${
                  i <= progress.stepIndex
                    ? "bg-ciab-copper"
                    : i === progress.stepIndex + 1
                      ? "bg-ciab-copper/30"
                      : "bg-ciab-bg-hover"
                }`}
                title={step.label}
              />
            ))}
          </div>

          {progress.status === "failed" && progress.error && (
            <p className="text-[10px] font-mono text-state-failed truncate">
              {progress.error}
            </p>
          )}
        </div>
      )}

      {/* Tabs */}
      <div className="flex items-center gap-0.5 border-b border-ciab-border mb-3 overflow-x-auto scrollbar-none">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 -mb-[1px] transition-colors ${
              activeTab === tab.id
                ? "border-ciab-copper text-ciab-copper"
                : "border-transparent text-ciab-text-muted hover:text-ciab-text-secondary"
            }`}
          >
            <tab.icon className="w-3.5 h-3.5" />
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="flex-1 min-h-0">
        {activeTab === "chat" && <ChatView sandboxId={sandbox.id} agentProvider={sandbox.agent_provider} agentConfig={sandbox.spec.agent_config} />}
        {activeTab === "terminal" && <TerminalView sandboxId={sandbox.id} />}
        {activeTab === "files" && <FileBrowser sandboxId={sandbox.id} />}
        {activeTab === "logs" && <LogViewer sandboxId={sandbox.id} />}
        {activeTab === "stats" && <StatsView sandboxId={sandbox.id} />}
      </div>
    </div>
  );
}
