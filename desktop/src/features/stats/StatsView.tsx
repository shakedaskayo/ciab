import { useSandboxStats } from "@/lib/hooks/use-sandboxes";
import { formatBytes } from "@/lib/utils/format";
import LoadingSpinner from "@/components/shared/LoadingSpinner";
import {
  Cpu,
  MemoryStick,
  HardDrive,
  ArrowDownToLine,
  ArrowUpFromLine,
  Activity,
  RefreshCw,
} from "lucide-react";

interface Props {
  sandboxId: string;
}

export default function StatsView({ sandboxId }: Props) {
  const { data: stats, isLoading, refetch, dataUpdatedAt } = useSandboxStats(sandboxId);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  if (!stats) {
    return (
      <div className="flex flex-col items-center justify-center h-64 text-center">
        <Activity className="w-8 h-8 text-ciab-text-muted/30 mb-3" />
        <p className="text-sm text-ciab-text-secondary">No statistics available</p>
        <p className="text-xs text-ciab-text-muted mt-1">
          Stats will appear when the sandbox is running
        </p>
      </div>
    );
  }

  const cpuPercent = Math.min(stats.cpu_usage_percent, 100);
  const memPercent =
    stats.memory_limit_mb > 0
      ? Math.min((stats.memory_used_mb / stats.memory_limit_mb) * 100, 100)
      : 0;
  const diskPercent =
    stats.disk_limit_mb > 0
      ? Math.min((stats.disk_used_mb / stats.disk_limit_mb) * 100, 100)
      : 0;

  const lastUpdated = dataUpdatedAt
    ? new Date(dataUpdatedAt).toLocaleTimeString("en-US", {
        hour12: false,
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      })
    : null;

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Activity className="w-4 h-4 text-ciab-text-muted" />
          <span className="text-xs font-mono text-ciab-text-muted uppercase tracking-wider">
            Resource Usage
          </span>
        </div>
        <div className="flex items-center gap-2">
          {lastUpdated && (
            <span className="text-[9px] font-mono text-ciab-text-muted/50">
              Updated {lastUpdated}
            </span>
          )}
          <button
            onClick={() => refetch()}
            className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
            title="Refresh stats"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Resource gauges */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
        <GaugeCard
          icon={Cpu}
          label="CPU"
          value={`${stats.cpu_usage_percent.toFixed(1)}%`}
          percent={cpuPercent}
          color="ciab-copper"
          level={cpuPercent > 80 ? "critical" : cpuPercent > 60 ? "warning" : "normal"}
        />
        <GaugeCard
          icon={MemoryStick}
          label="Memory"
          value={`${stats.memory_used_mb} MB`}
          subtitle={`of ${stats.memory_limit_mb} MB`}
          percent={memPercent}
          color="ciab-steel-blue"
          level={memPercent > 85 ? "critical" : memPercent > 70 ? "warning" : "normal"}
        />
        <GaugeCard
          icon={HardDrive}
          label="Disk"
          value={`${stats.disk_used_mb} MB`}
          subtitle={`of ${stats.disk_limit_mb} MB`}
          percent={diskPercent}
          color="state-paused"
          level={diskPercent > 90 ? "critical" : diskPercent > 75 ? "warning" : "normal"}
        />
      </div>

      {/* Network */}
      <div className="bg-ciab-bg-card rounded-xl border border-ciab-border p-4">
        <div className="flex items-center gap-2 mb-4">
          <Activity className="w-3.5 h-3.5 text-ciab-text-muted" />
          <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
            Network I/O
          </span>
        </div>
        <div className="grid grid-cols-2 gap-6">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-state-running/5 border border-state-running/10 flex items-center justify-center">
              <ArrowDownToLine className="w-4 h-4 text-state-running/70" />
            </div>
            <div>
              <p className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider mb-0.5">
                Received
              </p>
              <p className="text-lg font-semibold tabular-nums">
                {formatBytes(stats.network_rx_bytes)}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-ciab-copper/5 border border-ciab-copper/10 flex items-center justify-center">
              <ArrowUpFromLine className="w-4 h-4 text-ciab-copper/70" />
            </div>
            <div>
              <p className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider mb-0.5">
                Transmitted
              </p>
              <p className="text-lg font-semibold tabular-nums">
                {formatBytes(stats.network_tx_bytes)}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function GaugeCard({
  icon: Icon,
  label,
  value,
  subtitle,
  percent,
  color,
  level,
}: {
  icon: typeof Cpu;
  label: string;
  value: string;
  subtitle?: string;
  percent: number;
  color: string;
  level: "normal" | "warning" | "critical";
}) {
  const barColor =
    level === "critical"
      ? "bg-state-failed"
      : level === "warning"
        ? "bg-state-paused"
        : `bg-${color}`;

  const ringColor =
    level === "critical"
      ? "ring-state-failed/20"
      : level === "warning"
        ? "ring-state-paused/20"
        : `ring-${color}/20`;

  return (
    <div className="bg-ciab-bg-card rounded-xl border border-ciab-border p-4">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <div
            className={`w-7 h-7 rounded-lg bg-${color}/10 ${ringColor} ring-1 flex items-center justify-center`}
          >
            <Icon className={`w-3.5 h-3.5 text-${color}`} />
          </div>
          <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
            {label}
          </span>
        </div>
        {level !== "normal" && (
          <span
            className={`text-[9px] font-mono font-semibold uppercase px-1.5 py-0.5 rounded ${
              level === "critical"
                ? "bg-state-failed/10 text-state-failed"
                : "bg-state-paused/10 text-state-paused"
            }`}
          >
            {level === "critical" ? "HIGH" : "WARN"}
          </span>
        )}
      </div>

      <p className="text-2xl font-semibold tabular-nums">{value}</p>
      {subtitle && (
        <p className="text-[10px] text-ciab-text-muted font-mono mt-0.5">
          {subtitle}
        </p>
      )}

      {/* Progress bar */}
      <div className="mt-3 h-2 bg-ciab-bg-hover rounded-full overflow-hidden">
        <div
          className={`h-full ${barColor} rounded-full transition-all duration-700 ease-out`}
          style={{ width: `${percent}%` }}
        />
      </div>
      <div className="flex justify-between mt-1">
        <span className="text-[9px] text-ciab-text-muted/40 font-mono">0%</span>
        <span className="text-[9px] text-ciab-text-muted/40 font-mono">
          {percent.toFixed(0)}%
        </span>
        <span className="text-[9px] text-ciab-text-muted/40 font-mono">100%</span>
      </div>
    </div>
  );
}
