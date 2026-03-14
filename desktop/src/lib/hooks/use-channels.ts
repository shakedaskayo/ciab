import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { channels } from "@/lib/api/endpoints";
import type { CreateChannelRequest, UpdateChannelRequest } from "@/lib/api/types";
import { toast } from "sonner";

export function useChannels(params?: {
  provider?: string;
  state?: string;
  name?: string;
}) {
  return useQuery({
    queryKey: ["channels", params],
    queryFn: () => channels.list(params),
    refetchInterval: 10000,
  });
}

export function useChannel(id: string | undefined) {
  return useQuery({
    queryKey: ["channel", id],
    queryFn: () => channels.get(id!),
    enabled: !!id,
  });
}

export function useCreateChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (request: CreateChannelRequest) => channels.create(request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["channels"] });
      toast.success("Channel created");
    },
    onError: (err: Error) => {
      toast.error(`Failed to create channel: ${err.message}`);
    },
  });
}

export function useUpdateChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...request }: UpdateChannelRequest & { id: string }) =>
      channels.update(id, request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["channels"] });
      toast.success("Channel updated");
    },
    onError: (err: Error) => {
      toast.error(`Failed to update channel: ${err.message}`);
    },
  });
}

export function useDeleteChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => channels.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["channels"] });
      toast.success("Channel deleted");
    },
    onError: (err: Error) => {
      toast.error(`Failed to delete channel: ${err.message}`);
    },
  });
}

export function useStartChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => channels.start(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["channels"] });
      toast.success("Channel started");
    },
    onError: (err: Error) => {
      toast.error(`Failed to start channel: ${err.message}`);
    },
  });
}

export function useStopChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => channels.stop(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["channels"] });
      toast.success("Channel stopped");
    },
    onError: (err: Error) => {
      toast.error(`Failed to stop channel: ${err.message}`);
    },
  });
}

export function useRestartChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => channels.restart(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["channels"] });
      toast.success("Channel restarted");
    },
    onError: (err: Error) => {
      toast.error(`Failed to restart channel: ${err.message}`);
    },
  });
}

export function useChannelMessages(
  id: string | undefined,
  params?: { limit?: number; sender_id?: string }
) {
  return useQuery({
    queryKey: ["channel-messages", id, params],
    queryFn: () => channels.messages(id!, params),
    enabled: !!id,
    refetchInterval: 5000,
  });
}

export function useChannelQr(id: string | undefined, enabled: boolean) {
  return useQuery({
    queryKey: ["channel-qr", id],
    queryFn: () => channels.qr(id!),
    enabled: !!id && enabled,
    refetchInterval: 3000,
  });
}
