import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { workspaces } from "@/lib/api/endpoints";
import type { CreateWorkspaceRequest, UpdateWorkspaceRequest, WorkspaceSpec } from "@/lib/api/types";
import { toast } from "sonner";

export function useWorkspaces(name?: string) {
  return useQuery({
    queryKey: ["workspaces", name],
    queryFn: () => workspaces.list(name ? { name } : undefined),
    refetchInterval: 10000,
  });
}

export function useWorkspace(id: string | undefined) {
  return useQuery({
    queryKey: ["workspace", id],
    queryFn: () => workspaces.get(id!),
    enabled: !!id,
  });
}

export function useCreateWorkspace() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (request: CreateWorkspaceRequest) => workspaces.create(request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["workspaces"] });
      toast.success("Workspace created");
    },
    onError: (err: Error) => {
      toast.error(`Failed to create workspace: ${err.message}`);
    },
  });
}

export function useUpdateWorkspace() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...request }: UpdateWorkspaceRequest & { id: string }) =>
      workspaces.update(id, request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["workspaces"] });
      toast.success("Workspace updated");
    },
    onError: (err: Error) => {
      toast.error(`Failed to update workspace: ${err.message}`);
    },
  });
}

export function useDeleteWorkspace() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => workspaces.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["workspaces"] });
      toast.success("Workspace deleted");
    },
    onError: (err: Error) => {
      toast.error(`Failed to delete workspace: ${err.message}`);
    },
  });
}

export function useLaunchWorkspace() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, spec_overrides }: { id: string; spec_overrides?: Partial<WorkspaceSpec> }) =>
      workspaces.launch(id, spec_overrides),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sandboxes"] });
      toast.success("Workspace launched — sandbox provisioning started");
    },
    onError: (err: Error) => {
      toast.error(`Failed to launch workspace: ${err.message}`);
    },
  });
}

export function useWorkspaceSandboxes(id: string | undefined) {
  return useQuery({
    queryKey: ["workspace-sandboxes", id],
    queryFn: () => workspaces.sandboxes(id!),
    enabled: !!id,
  });
}
