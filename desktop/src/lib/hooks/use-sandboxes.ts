import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { sandboxes } from "@/lib/api/endpoints";
import type { SandboxSpec } from "@/lib/api/types";
import { toast } from "sonner";

export function useSandboxes(params?: { state?: string; provider?: string }) {
  return useQuery({
    queryKey: ["sandboxes", params],
    queryFn: () => sandboxes.list(params),
    refetchInterval: (query) => {
      // Poll faster when there are provisioning sandboxes
      const data = query.state.data as import("@/lib/api/types").SandboxInfo[] | undefined;
      const hasProvisioning = data?.some(
        (s) => s.state === "creating" || s.state === "pending"
      );
      return hasProvisioning ? 3000 : 10000;
    },
  });
}

export function useSandbox(id: string) {
  return useQuery({
    queryKey: ["sandbox", id],
    queryFn: () => sandboxes.get(id),
    enabled: !!id,
    refetchInterval: 5000,
  });
}

export function useSandboxStats(id: string) {
  return useQuery({
    queryKey: ["sandbox-stats", id],
    queryFn: () => sandboxes.stats(id),
    enabled: !!id,
    refetchInterval: 3000,
  });
}

export function useCreateSandbox() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (spec: SandboxSpec) => sandboxes.create(spec),
    onSuccess: (result) => {
      // refetchQueries forces an immediate refetch (not just mark stale)
      qc.refetchQueries({ queryKey: ["sandboxes"] });
      toast.success(`Sandbox provisioning started (${result.sandbox_id.slice(0, 8)})`);
    },
    onError: (error) => {
      toast.error(`Failed to create sandbox: ${error.message}`);
    },
  });
}

export function useDeleteSandbox() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => sandboxes.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sandboxes"] });
      toast.success("Sandbox deleted");
    },
    onError: (error) => {
      toast.error(`Failed to delete sandbox: ${error.message}`);
    },
  });
}

export function useSandboxAction() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      action,
    }: {
      id: string;
      action: "start" | "stop" | "pause" | "resume";
    }) => sandboxes[action](id),
    onSuccess: (_, { id, action }) => {
      qc.invalidateQueries({ queryKey: ["sandbox", id] });
      qc.invalidateQueries({ queryKey: ["sandboxes"] });
      toast.success(`Sandbox ${action} initiated`);
    },
    onError: (error) => {
      toast.error(`Action failed: ${error.message}`);
    },
  });
}
