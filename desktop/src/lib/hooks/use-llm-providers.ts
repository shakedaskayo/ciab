import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { llmProviders } from "@/lib/api/endpoints";
import type { CreateLlmProviderRequest, UpdateLlmProviderRequest } from "@/lib/api/types";
import { toast } from "sonner";

export function useLlmProviders() {
  return useQuery({
    queryKey: ["llm-providers"],
    queryFn: () => llmProviders.list(),
    refetchInterval: 30000,
  });
}

export function useLlmProvider(id: string | undefined) {
  return useQuery({
    queryKey: ["llm-provider", id],
    queryFn: () => llmProviders.get(id!),
    enabled: !!id,
  });
}

export function useLlmProviderModels(id: string | undefined) {
  return useQuery({
    queryKey: ["llm-provider-models", id],
    queryFn: () => llmProviders.models(id!),
    enabled: !!id,
  });
}

export function useCreateLlmProvider() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (request: CreateLlmProviderRequest) => llmProviders.create(request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["llm-providers"] });
      toast.success("LLM provider created");
    },
    onError: (err: Error) => {
      toast.error(`Failed to create LLM provider: ${err.message}`);
    },
  });
}

export function useUpdateLlmProvider() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...request }: UpdateLlmProviderRequest & { id: string }) =>
      llmProviders.update(id, request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["llm-providers"] });
      toast.success("LLM provider updated");
    },
    onError: (err: Error) => {
      toast.error(`Failed to update LLM provider: ${err.message}`);
    },
  });
}

export function useDeleteLlmProvider() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => llmProviders.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["llm-providers"] });
      toast.success("LLM provider deleted");
    },
    onError: (err: Error) => {
      toast.error(`Failed to delete LLM provider: ${err.message}`);
    },
  });
}

export function useTestLlmProvider() {
  return useMutation({
    mutationFn: (id: string) => llmProviders.test(id),
  });
}

export function useDetectLlmProviders() {
  return useMutation({
    mutationFn: () => llmProviders.detect(),
  });
}

export function useRefreshModels() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => llmProviders.refreshModels(id),
    onSuccess: (_data, id) => {
      qc.invalidateQueries({ queryKey: ["llm-provider-models", id] });
      toast.success("Models refreshed");
    },
    onError: (err: Error) => {
      toast.error(`Failed to refresh models: ${err.message}`);
    },
  });
}

export function useOllamaPull() {
  return useMutation({
    mutationFn: ({ model, base_url }: { model: string; base_url?: string }) =>
      llmProviders.ollamaPull(model, base_url),
    onSuccess: (data) => {
      toast.success(`Model pull complete: ${data.status}`);
    },
    onError: (err: Error) => {
      toast.error(`Failed to pull model: ${err.message}`);
    },
  });
}

export function useCompatibility() {
  return useQuery({
    queryKey: ["llm-compatibility"],
    queryFn: () => llmProviders.compatibility(),
    staleTime: 60000,
  });
}
