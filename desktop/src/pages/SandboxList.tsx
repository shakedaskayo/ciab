import { useState, useMemo } from "react";
import { useNavigate } from "react-router";
import { useSandboxes } from "@/lib/hooks/use-sandboxes";
import SandboxGrid from "@/features/dashboard/SandboxGrid";
import CreateSandboxDialog from "@/features/sandbox/CreateSandboxDialog";
import LoadingSpinner from "@/components/shared/LoadingSpinner";
import EmptyState from "@/components/shared/EmptyState";
import {
  Container,
  Plus,
  RefreshCw,
  CircleDot,
  Loader2,
  Square,
  AlertTriangle,
  LayoutGrid,
} from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import type { SandboxState } from "@/lib/api/types";

type FilterTab = "all" | "running" | "provisioning" | "stopped" | "failed";

const FILTER_TABS: Array<{
  id: FilterTab;
  label: string;
  icon: typeof CircleDot;
  states: SandboxState[];
}> = [
  { id: "all", label: "All", icon: LayoutGrid, states: [] },
  { id: "running", label: "Running", icon: CircleDot, states: ["running"] },
  {
    id: "provisioning",
    label: "Provisioning",
    icon: Loader2,
    states: ["creating", "pending"],
  },
  {
    id: "stopped",
    label: "Stopped",
    icon: Square,
    states: ["stopped", "stopping", "terminated", "paused", "pausing"],
  },
  {
    id: "failed",
    label: "Failed",
    icon: AlertTriangle,
    states: ["failed"],
  },
];

export default function SandboxList() {
  const [showCreate, setShowCreate] = useState(false);
  const [filter, setFilter] = useState<FilterTab>("all");
  const { data: sandboxList, isLoading, isFetching } = useSandboxes();
  const qc = useQueryClient();
  const navigate = useNavigate();
  const [spinning, setSpinning] = useState(false);

  const handleRefresh = () => {
    setSpinning(true);
    qc.invalidateQueries({ queryKey: ["sandboxes"] });
    setTimeout(() => setSpinning(false), 600);
  };

  // Count sandboxes per filter tab
  const counts = useMemo(() => {
    const c: Record<FilterTab, number> = {
      all: 0,
      running: 0,
      provisioning: 0,
      stopped: 0,
      failed: 0,
    };
    if (!sandboxList) return c;
    c.all = sandboxList.length;
    for (const s of sandboxList) {
      for (const tab of FILTER_TABS) {
        if (tab.id !== "all" && tab.states.includes(s.state)) {
          c[tab.id]++;
        }
      }
    }
    return c;
  }, [sandboxList]);

  // Filter and sort
  const filtered = useMemo(() => {
    if (!sandboxList) return [];
    const tab = FILTER_TABS.find((t) => t.id === filter)!;
    let list =
      tab.id === "all"
        ? [...sandboxList]
        : sandboxList.filter((s) => tab.states.includes(s.state));

    // Sort: provisioning first, then running, then by newest first
    list.sort((a, b) => {
      const priority = (state: SandboxState) => {
        if (state === "creating" || state === "pending") return 0;
        if (state === "running") return 1;
        if (state === "failed") return 2;
        return 3;
      };
      const pa = priority(a.state);
      const pb = priority(b.state);
      if (pa !== pb) return pa - pb;
      return (
        new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
      );
    });

    return list;
  }, [sandboxList, filter]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="space-y-4 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">Sandboxes</h1>
          <p className="text-sm text-ciab-text-muted mt-0.5">
            {sandboxList?.length ?? 0} total
            {counts.running > 0 && (
              <span className="text-state-running">
                {" "}
                &middot; {counts.running} running
              </span>
            )}
            {counts.provisioning > 0 && (
              <span className="text-state-creating">
                {" "}
                &middot; {counts.provisioning} provisioning
              </span>
            )}
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
            onClick={() => setShowCreate(true)}
            className="btn-primary flex items-center gap-2"
          >
            <Plus className="w-4 h-4" />
            New Sandbox
          </button>
        </div>
      </div>

      {/* Filter tabs */}
      {(sandboxList?.length ?? 0) > 0 && (
        <div className="flex items-center gap-1 overflow-x-auto scrollbar-none">
          {FILTER_TABS.map((tab) => {
            const count = counts[tab.id];
            const active = filter === tab.id;
            // Hide tabs with 0 count (except All)
            if (tab.id !== "all" && count === 0) return null;
            return (
              <button
                key={tab.id}
                onClick={() => setFilter(tab.id)}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all whitespace-nowrap ${
                  active
                    ? "bg-ciab-copper/10 text-ciab-copper shadow-sm"
                    : "text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover"
                }`}
              >
                <tab.icon
                  className={`w-3.5 h-3.5 ${tab.id === "provisioning" && active ? "animate-spin" : ""}`}
                />
                {tab.label}
                <span
                  className={`text-[10px] font-mono px-1.5 py-0.5 rounded-full ${
                    active
                      ? "bg-ciab-copper/15 text-ciab-copper"
                      : "bg-ciab-bg-elevated text-ciab-text-muted"
                  }`}
                >
                  {count}
                </span>
              </button>
            );
          })}
        </div>
      )}

      {/* Grid */}
      {filtered.length > 0 ? (
        <SandboxGrid sandboxes={filtered} />
      ) : sandboxList && sandboxList.length > 0 ? (
        <EmptyState
          icon={Container}
          title={`No ${filter} sandboxes`}
          description="Try a different filter or create a new sandbox."
        />
      ) : (
        <EmptyState
          icon={Container}
          title="No sandboxes"
          description="Create a sandbox to start working with a coding agent."
          action={
            <button
              onClick={() => setShowCreate(true)}
              className="btn-primary"
            >
              Create Sandbox
            </button>
          }
        />
      )}

      {showCreate && (
        <CreateSandboxDialog
          onClose={() => setShowCreate(false)}
          onSuccess={(sandboxId) => {
            setShowCreate(false);
            navigate(`/sandboxes/${sandboxId}`);
          }}
        />
      )}
    </div>
  );
}
