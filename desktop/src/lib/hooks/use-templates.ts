import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { templates } from "@/lib/api/endpoints";
import type { CreateFromTemplateRequest, AddTemplateSourceRequest } from "@/lib/api/types";
import { toast } from "sonner";

export function useTemplates() {
  return useQuery({
    queryKey: ["templates"],
    queryFn: () => templates.list(),
    refetchInterval: 10000,
  });
}

export function useCreateFromTemplate() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ templateId, ...request }: CreateFromTemplateRequest & { templateId: string }) =>
      templates.createFromTemplate(templateId, request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["workspaces"] });
      toast.success("Workspace created from template");
    },
    onError: (err: Error) => {
      toast.error(`Failed to create from template: ${err.message}`);
    },
  });
}

export function useTemplateSources() {
  return useQuery({
    queryKey: ["template-sources"],
    queryFn: () => templates.listSources(),
  });
}

export function useAddTemplateSource() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (request: AddTemplateSourceRequest) => templates.addSource(request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["template-sources"] });
      toast.success("Template source added");
    },
    onError: (err: Error) => {
      toast.error(`Failed to add source: ${err.message}`);
    },
  });
}

export function useDeleteTemplateSource() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => templates.deleteSource(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["template-sources"] });
      qc.invalidateQueries({ queryKey: ["templates"] });
      toast.success("Template source removed");
    },
    onError: (err: Error) => {
      toast.error(`Failed to remove source: ${err.message}`);
    },
  });
}

export function useSyncTemplateSource() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => templates.syncSource(id),
    onSuccess: (data) => {
      qc.invalidateQueries({ queryKey: ["templates"] });
      qc.invalidateQueries({ queryKey: ["template-sources"] });
      toast.success(`Synced ${(data as { synced: number }).synced} templates`);
    },
    onError: (err: Error) => {
      toast.error(`Sync failed: ${err.message}`);
    },
  });
}
