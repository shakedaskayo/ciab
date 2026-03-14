import { useNavigate } from "react-router";
import type { SandboxInfo } from "@/lib/api/types";
import SandboxStateBadge from "@/components/shared/SandboxStateBadge";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";
import { truncateId, formatRelativeTime } from "@/lib/utils/format";
import { Play, Square, Pause, RotateCcw, Trash2 } from "lucide-react";
import { useSandboxAction, useDeleteSandbox } from "@/lib/hooks/use-sandboxes";
import { getAvailableActions } from "@/lib/utils/sandbox-state";
import {
  useProvisioningProgress,
  PROVISIONING_STEPS,
} from "@/lib/hooks/use-provisioning";

interface Props {
  sandbox: SandboxInfo;
}

const actionIcons = {
  start: Play,
  stop: Square,
  pause: Pause,
  resume: RotateCcw,
  delete: Trash2,
};

const PROVIDER_LABELS: Record<string, string> = {
  "claude-code": "Claude Code",
  codex: "Codex",
  gemini: "Gemini CLI",
  cursor: "Cursor",
};

export default function SandboxCard({ sandbox }: Props) {
  const navigate = useNavigate();
  const sandboxAction = useSandboxAction();
  const deleteSandbox = useDeleteSandbox();
  const actions = getAvailableActions(sandbox.state);

  const isProvisioning =
    sandbox.state === "creating" || sandbox.state === "pending";

  const progress = useProvisioningProgress(sandbox.id, isProvisioning);

  return (
    <div
      className="card-hover group p-0 overflow-hidden animate-fade-in"
      onClick={() => navigate(`/sandboxes/${sandbox.id}`)}
    >
      {/* Top accent bar — color by provider, animated when provisioning */}
      {isProvisioning ? (
        <div className="h-[2px] bg-ciab-border relative overflow-hidden">
          <div
            className="absolute inset-y-0 left-0 transition-all duration-700 ease-out"
            style={{
              width: `${progress.percent}%`,
              background:
                sandbox.agent_provider === "claude-code"
                  ? "#D97757"
                  : sandbox.agent_provider === "codex"
                    ? "#10A37F"
                    : sandbox.agent_provider === "gemini"
                      ? "#4285F4"
                      : sandbox.agent_provider === "cursor"
                        ? "#7C3AED"
                        : "#C4693D",
            }}
          />
        </div>
      ) : (
        <div
          className={`h-[2px] ${
            sandbox.agent_provider === "claude-code"
              ? "bg-provider-claude"
              : sandbox.agent_provider === "codex"
                ? "bg-provider-openai"
                : sandbox.agent_provider === "gemini"
                  ? "bg-provider-gemini"
                  : sandbox.agent_provider === "cursor"
                    ? "bg-provider-cursor"
                    : "bg-ciab-border-light"
          }`}
        />
      )}

      <div className="p-3.5">
        <div className="flex items-start justify-between mb-3">
          <div className="flex items-center gap-2.5 min-w-0">
            <div className="flex-shrink-0 w-8 h-8 rounded-md bg-ciab-bg-hover flex items-center justify-center">
              <AgentProviderIcon provider={sandbox.agent_provider} size={18} />
            </div>
            <div className="min-w-0">
              <p className="font-medium text-sm text-ciab-text-primary truncate leading-tight">
                {sandbox.name ?? truncateId(sandbox.id)}
              </p>
              <p className="text-[10px] font-mono text-ciab-text-muted mt-0.5">
                {PROVIDER_LABELS[sandbox.agent_provider] ??
                  sandbox.agent_provider}
              </p>
            </div>
          </div>
          <SandboxStateBadge state={sandbox.state} />
        </div>

        {/* Provisioning progress section */}
        {isProvisioning && (
          <div className="mb-3 space-y-2">
            {/* Progress bar */}
            <div className="relative">
              <div className="h-1.5 bg-ciab-bg-hover rounded-full overflow-hidden">
                <div
                  className="h-full rounded-full transition-all duration-700 ease-out bg-gradient-to-r from-ciab-copper/80 to-ciab-copper"
                  style={{ width: `${Math.max(progress.percent, 3)}%` }}
                />
              </div>
            </div>

            {/* Current step info */}
            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-1.5 min-w-0">
                <div className="w-1 h-1 rounded-full bg-ciab-copper animate-pulse" />
                <span className="text-[10px] font-mono text-ciab-text-secondary truncate">
                  {progress.currentStepLabel ?? "Initializing..."}
                </span>
              </div>
              <span className="text-[10px] font-mono text-ciab-text-muted flex-shrink-0">
                {progress.stepIndex >= 0
                  ? `${progress.stepIndex + 1}/${progress.totalSteps}`
                  : ""}
              </span>
            </div>

            {/* Step dots */}
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

            {/* Optional message */}
            {progress.message && (
              <p className="text-[9px] font-mono text-ciab-text-muted truncate">
                {progress.message}
              </p>
            )}

            {/* Error state */}
            {progress.status === "failed" && progress.error && (
              <p className="text-[9px] font-mono text-state-failed truncate">
                {progress.error}
              </p>
            )}
          </div>
        )}

        <div className="flex items-center justify-between">
          <span className="text-[11px] text-ciab-text-muted font-mono">
            {truncateId(sandbox.id)} &middot;{" "}
            {formatRelativeTime(sandbox.created_at)}
          </span>

          <div
            className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
            onClick={(e) => e.stopPropagation()}
          >
            {actions.map((action) => {
              const Icon = actionIcons[action];
              return (
                <button
                  key={action}
                  onClick={() =>
                    action === "delete"
                      ? deleteSandbox.mutate(sandbox.id)
                      : sandboxAction.mutate({ id: sandbox.id, action })
                  }
                  className={`p-1.5 rounded transition-colors ${
                    action === "delete"
                      ? "text-state-failed/70 hover:text-state-failed hover:bg-state-failed/10"
                      : "text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover"
                  }`}
                  title={action}
                >
                  <Icon className="w-3.5 h-3.5" />
                </button>
              );
            })}
          </div>
        </div>

        {/* Resource bar — only when running */}
        {sandbox.resource_stats &&
          sandbox.resource_stats.memory_limit_mb > 0 &&
          !isProvisioning && (
            <div className="mt-3 pt-3 border-t border-ciab-border">
              <div className="flex items-center justify-between text-[10px] font-mono text-ciab-text-muted mb-1">
                <span>
                  CPU {sandbox.resource_stats.cpu_usage_percent.toFixed(0)}%
                </span>
                <span>
                  {sandbox.resource_stats.memory_used_mb}/
                  {sandbox.resource_stats.memory_limit_mb}MB
                </span>
              </div>
              <div className="h-1 bg-ciab-bg-hover rounded-full overflow-hidden">
                <div
                  className="h-full bg-ciab-copper/70 rounded-full transition-all duration-500"
                  style={{
                    width: `${Math.min(
                      (sandbox.resource_stats.memory_used_mb /
                        sandbox.resource_stats.memory_limit_mb) *
                        100,
                      100
                    )}%`,
                  }}
                />
              </div>
            </div>
          )}
      </div>
    </div>
  );
}
