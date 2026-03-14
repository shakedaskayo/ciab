import type { SandboxState } from "@/lib/api/types";

export function getStateColor(state: SandboxState): string {
  switch (state) {
    case "running":
      return "text-state-running bg-state-running/10";
    case "paused":
    case "pausing":
      return "text-state-paused bg-state-paused/10";
    case "failed":
      return "text-state-failed bg-state-failed/10";
    case "stopped":
    case "stopping":
    case "terminated":
      return "text-state-stopped bg-state-stopped/10";
    case "creating":
    case "pending":
      return "text-state-creating bg-state-creating/10";
    default:
      return "text-ciab-text-secondary bg-ciab-bg-hover";
  }
}

export function getStateDotColor(state: SandboxState): string {
  switch (state) {
    case "running":
      return "bg-state-running";
    case "paused":
    case "pausing":
      return "bg-state-paused";
    case "failed":
      return "bg-state-failed";
    case "stopped":
    case "stopping":
    case "terminated":
      return "bg-state-stopped";
    case "creating":
    case "pending":
      return "bg-state-creating";
    default:
      return "bg-ciab-text-muted";
  }
}

export function isActionable(state: SandboxState): boolean {
  return ["running", "paused", "stopped"].includes(state);
}

export function getAvailableActions(
  state: SandboxState
): Array<"start" | "stop" | "pause" | "resume" | "delete"> {
  switch (state) {
    case "running":
      return ["stop", "pause", "delete"];
    case "paused":
      return ["resume", "stop", "delete"];
    case "stopped":
      return ["start", "delete"];
    case "failed":
    case "terminated":
      return ["delete"];
    default:
      return [];
  }
}
