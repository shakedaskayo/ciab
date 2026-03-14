import { useMutation } from "@tanstack/react-query";
import { exec } from "@/lib/api/endpoints";
import type { ExecRequest } from "@/lib/api/types";

export function useExec(sandboxId: string) {
  return useMutation({
    mutationFn: (request: ExecRequest) => exec.run(sandboxId, request),
  });
}
