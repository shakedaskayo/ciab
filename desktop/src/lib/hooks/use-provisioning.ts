import { useState, useEffect, useRef } from "react";
import { subscribeSandboxStream } from "@/lib/api/sse";
import type { StreamEvent } from "@/lib/api/types";

const PROVISIONING_STEPS = [
  { id: "validate", label: "Validating spec" },
  { id: "prepare_image", label: "Preparing image" },
  { id: "resolve_credentials", label: "Resolving credentials" },
  { id: "create_sandbox", label: "Creating sandbox" },
  { id: "start_sandbox", label: "Starting sandbox" },
  { id: "inject_credentials", label: "Injecting credentials" },
  { id: "clone_repositories", label: "Cloning repos" },
  { id: "run_scripts", label: "Running scripts" },
  { id: "start_agent", label: "Starting agent" },
] as const;

export interface ProvisioningProgress {
  currentStep: string | null;
  currentStepLabel: string | null;
  stepIndex: number;
  totalSteps: number;
  percent: number;
  message: string | null;
  status: "provisioning" | "complete" | "failed";
  error: string | null;
}

/**
 * Subscribe to a sandbox SSE stream and track provisioning progress.
 * Only active when `enabled` is true (i.e. sandbox is in creating/pending state).
 */
export function useProvisioningProgress(
  sandboxId: string,
  enabled: boolean
): ProvisioningProgress {
  const [progress, setProgress] = useState<ProvisioningProgress>({
    currentStep: null,
    currentStepLabel: null,
    stepIndex: -1,
    totalSteps: PROVISIONING_STEPS.length,
    percent: 0,
    message: null,
    status: "provisioning",
    error: null,
  });

  const connectionRef = useRef<{ close: () => void } | null>(null);

  useEffect(() => {
    if (!enabled || !sandboxId) {
      connectionRef.current?.close();
      connectionRef.current = null;
      return;
    }

    const handleEvent = (event: StreamEvent) => {
      if (event.event_type === "provisioning_step") {
        const data = event.data as {
          step?: string;
          status?: string;
          message?: string;
        };
        const stepId = data.step ?? null;
        const idx = stepId
          ? PROVISIONING_STEPS.findIndex((s) => s.id === stepId)
          : -1;

        setProgress((prev) => ({
          ...prev,
          currentStep: stepId,
          currentStepLabel:
            idx >= 0 ? PROVISIONING_STEPS[idx].label : stepId,
          stepIndex: idx >= 0 ? idx : prev.stepIndex,
          percent:
            idx >= 0
              ? Math.round(((idx + 1) / PROVISIONING_STEPS.length) * 100)
              : prev.percent,
          message: data.message ?? prev.message,
          status: "provisioning",
        }));
      } else if (event.event_type === "provisioning_complete") {
        setProgress((prev) => ({
          ...prev,
          percent: 100,
          stepIndex: PROVISIONING_STEPS.length - 1,
          currentStepLabel: "Complete",
          status: "complete",
        }));
      } else if (event.event_type === "provisioning_failed") {
        const data = event.data as { error?: string };
        setProgress((prev) => ({
          ...prev,
          status: "failed",
          error: data.error ?? "Provisioning failed",
        }));
      }
    };

    connectionRef.current = subscribeSandboxStream(sandboxId, handleEvent);

    return () => {
      connectionRef.current?.close();
      connectionRef.current = null;
    };
  }, [sandboxId, enabled]);

  return progress;
}

export { PROVISIONING_STEPS };
