import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { credentials } from "@/lib/api/endpoints";
import type { CreateCredentialRequest } from "@/lib/api/types";
import { toast } from "sonner";

export function useCredentials() {
  return useQuery({
    queryKey: ["credentials"],
    queryFn: () => credentials.list(),
  });
}

export function useCreateCredential() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (request: CreateCredentialRequest) =>
      credentials.create(request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["credentials"] });
      toast.success("Credential created");
    },
    onError: (error) => {
      toast.error(`Failed to create credential: ${error.message}`);
    },
  });
}

export function useDeleteCredential() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => credentials.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["credentials"] });
      toast.success("Credential deleted");
    },
    onError: (error) => {
      toast.error(`Failed to delete credential: ${error.message}`);
    },
  });
}
