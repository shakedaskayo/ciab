import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { files } from "@/lib/api/endpoints";
import { toast } from "sonner";

export function useFileList(sandboxId: string, path: string = "/") {
  return useQuery({
    queryKey: ["files", sandboxId, path],
    queryFn: () => files.list(sandboxId, path),
    enabled: !!sandboxId,
  });
}

export function useUploadFile(sandboxId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ path, data }: { path: string; data: ArrayBuffer }) =>
      files.upload(sandboxId, path, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["files", sandboxId] });
      toast.success("File uploaded");
    },
    onError: (error) => {
      toast.error(`Upload failed: ${error.message}`);
    },
  });
}

export function useDeleteFile(sandboxId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (path: string) => files.delete(sandboxId, path),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["files", sandboxId] });
      toast.success("File deleted");
    },
  });
}
