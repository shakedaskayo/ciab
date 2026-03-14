import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { gateway } from "@/lib/api/endpoints";
import type { CreateTokenRequest, CreateTunnelRequest, ExposeRequest, UpdateGatewayConfigRequest, ProviderPrepareResult } from "@/lib/api/types";
import { toast } from "sonner";

export function useGatewayStatus() {
  return useQuery({
    queryKey: ["gateway-status"],
    queryFn: () => gateway.status(),
    refetchInterval: 15000,
    retry: false,
  });
}

export function useGatewayConfig() {
  return useQuery({
    queryKey: ["gateway-config"],
    queryFn: () => gateway.getConfig(),
    retry: false,
  });
}

export function useUpdateGatewayConfig() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (config: UpdateGatewayConfigRequest) => gateway.updateConfig(config),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["gateway-config"] });
      qc.invalidateQueries({ queryKey: ["gateway-status"] });
      toast.success("Gateway configuration updated");
    },
    onError: (err: Error) => {
      toast.error(`Failed to update gateway config: ${err.message}`);
    },
  });
}

export function useGatewayDiscover() {
  return useQuery({
    queryKey: ["gateway-discover"],
    queryFn: () => gateway.discover(),
    enabled: false, // manual trigger
  });
}

export function useGatewayTokens() {
  return useQuery({
    queryKey: ["gateway-tokens"],
    queryFn: () => gateway.listTokens(),
    refetchInterval: 10000,
  });
}

export function useCreateGatewayToken() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (request: CreateTokenRequest) => gateway.createToken(request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["gateway-tokens"] });
      qc.invalidateQueries({ queryKey: ["gateway-status"] });
    },
    onError: (err: Error) => {
      toast.error(`Failed to create token: ${err.message}`);
    },
  });
}

export function useRevokeGatewayToken() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => gateway.revokeToken(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["gateway-tokens"] });
      qc.invalidateQueries({ queryKey: ["gateway-status"] });
      toast.success("Token revoked");
    },
    onError: (err: Error) => {
      toast.error(`Failed to revoke token: ${err.message}`);
    },
  });
}

export function useGatewayTunnels() {
  return useQuery({
    queryKey: ["gateway-tunnels"],
    queryFn: () => gateway.listTunnels(),
    refetchInterval: 10000,
  });
}

export function useCreateGatewayTunnel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (request: CreateTunnelRequest) => gateway.createTunnel(request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["gateway-tunnels"] });
      qc.invalidateQueries({ queryKey: ["gateway-status"] });
      toast.success("Tunnel created");
    },
    onError: (err: Error) => {
      toast.error(`Failed to create tunnel: ${err.message}`);
    },
  });
}

export function useDeleteGatewayTunnel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => gateway.deleteTunnel(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["gateway-tunnels"] });
      qc.invalidateQueries({ queryKey: ["gateway-status"] });
      toast.success("Tunnel stopped");
    },
    onError: (err: Error) => {
      toast.error(`Failed to stop tunnel: ${err.message}`);
    },
  });
}

export function usePrepareProvider() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (provider: string) => gateway.prepareProvider(provider),
    onSuccess: (data: ProviderPrepareResult) => {
      qc.invalidateQueries({ queryKey: ["gateway-status"] });
      toast.success(data.message || `${data.provider} prepared successfully`);
    },
    onError: (err: Error) => {
      toast.error(`Failed to prepare provider: ${err.message}`);
    },
  });
}

export function useExposeGateway() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (request: ExposeRequest) => gateway.expose(request),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["gateway-tunnels"] });
      qc.invalidateQueries({ queryKey: ["gateway-tokens"] });
      qc.invalidateQueries({ queryKey: ["gateway-status"] });
    },
    onError: (err: Error) => {
      toast.error(`Failed to expose sandbox: ${err.message}`);
    },
  });
}
