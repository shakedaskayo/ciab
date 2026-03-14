import { Zap, Shield, Lock, Map, Unlock } from "lucide-react";
import type { PermissionMode } from "@/lib/api/types";

interface Props {
  mode: PermissionMode;
  onChange: (mode: PermissionMode) => void;
}

const MODES: Array<{
  value: PermissionMode;
  label: string;
  icon: typeof Zap;
  title: string;
  activeColor: string;
}> = [
  {
    value: "auto_approve",
    label: "Auto",
    icon: Zap,
    title: "Auto-approve all tool calls",
    activeColor: "bg-ciab-copper/15 text-ciab-copper shadow-sm",
  },
  {
    value: "approve_edits",
    label: "Safe",
    icon: Shield,
    title: "Approve edits & commands only",
    activeColor: "bg-emerald-500/15 text-emerald-400 shadow-sm",
  },
  {
    value: "approve_all",
    label: "Strict",
    icon: Lock,
    title: "Approve every tool call",
    activeColor: "bg-amber-500/15 text-amber-400 shadow-sm",
  },
  {
    value: "plan_only",
    label: "Plan",
    icon: Map,
    title: "Read-only planning mode",
    activeColor: "bg-ciab-steel-blue/15 text-ciab-steel-blue shadow-sm",
  },
  {
    value: "unrestricted",
    label: "Full",
    icon: Unlock,
    title: "Skip all permission checks — unrestricted",
    activeColor: "bg-red-500/15 text-red-400 shadow-sm",
  },
];

export default function PermissionModeSelector({ mode, onChange }: Props) {
  return (
    <div className="flex items-center bg-ciab-bg-secondary rounded-lg border border-ciab-border p-0.5 mr-1 flex-shrink-0">
      {MODES.map((m) => {
        const active = mode === m.value;
        return (
          <button
            key={m.value}
            onClick={() => onChange(m.value)}
            className={`flex items-center gap-1 px-2 py-1 rounded-md text-[10px] font-mono font-medium transition-all ${
              active
                ? m.activeColor
                : "text-ciab-text-muted hover:text-ciab-text-secondary"
            }`}
            title={m.title}
          >
            <m.icon className="w-2.5 h-2.5" />
            {m.label}
          </button>
        );
      })}
    </div>
  );
}
