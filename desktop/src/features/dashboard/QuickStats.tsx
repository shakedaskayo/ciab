import type { SandboxInfo } from "@/lib/api/types";

interface Props {
  sandboxes: SandboxInfo[];
}

export default function QuickStats({ sandboxes }: Props) {
  const total = sandboxes.length;
  const running = sandboxes.filter((s) => s.state === "running").length;
  const paused = sandboxes.filter(
    (s) => s.state === "paused" || s.state === "pausing"
  ).length;
  const failed = sandboxes.filter((s) => s.state === "failed").length;

  const stats = [
    { label: "TOTAL", value: total, color: "text-ciab-text-primary", dotColor: "bg-ciab-steel-blue" },
    { label: "RUNNING", value: running, color: "text-state-running", dotColor: "bg-state-running" },
    { label: "PAUSED", value: paused, color: "text-state-paused", dotColor: "bg-state-paused" },
    { label: "FAILED", value: failed, color: "text-state-failed", dotColor: "bg-state-failed" },
  ];

  return (
    <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
      {stats.map((stat) => (
        <div key={stat.label} className="card p-3">
          <div className="flex items-center gap-1.5 mb-2">
            <div className={`w-1.5 h-1.5 rounded-full ${stat.dotColor}`} />
            <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
              {stat.label}
            </span>
          </div>
          <p className={`text-2xl font-semibold tabular-nums ${stat.color}`}>
            {stat.value}
          </p>
        </div>
      ))}
    </div>
  );
}
