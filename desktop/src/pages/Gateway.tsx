import { useState, useMemo } from "react";
import { QRCodeSVG } from "qrcode.react";
import {
  Globe,
  Plus,
  Trash2,
  Copy,
  Check,
  Wifi,
  WifiOff,
  Key,
  Link2,
  Radio,
  Shield,
  X,
  Zap,
  Search,
  Settings2,
  ChevronDown,
  ChevronRight,
  Power,
  RefreshCw,
  Server,
  Network,
  AlertTriangle,
  Eye,
  EyeOff,
  Layers,
  Terminal,
  Download,
  Cloud,
  CheckCircle2,
  XCircle,
  Smartphone,
  ExternalLink,
  QrCode,
} from "lucide-react";
import {
  useGatewayStatus,
  useGatewayConfig,
  useUpdateGatewayConfig,
  useGatewayTokens,
  useGatewayTunnels,
  useCreateGatewayToken,
  useRevokeGatewayToken,
  useDeleteGatewayTunnel,
  useExposeGateway,
  usePrepareProvider,
} from "@/lib/hooks/use-gateway";
import { useSandboxes } from "@/lib/hooks/use-sandboxes";
import { useWorkspaces } from "@/lib/hooks/use-workspaces";
import { formatRelativeTime, truncateId } from "@/lib/utils/format";
import SandboxStateBadge from "@/components/shared/SandboxStateBadge";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";
import type {
  ClientToken,
  GatewayTunnel,
  UpdateGatewayConfigRequest,
  TokenScope,
  TokenScopeType,
  CreateTokenRequest,
  ExposeRequest,
} from "@/lib/api/types";
import { useQueryClient } from "@tanstack/react-query";

// =============================================================================
// Provider metadata
// =============================================================================

const PROVIDERS = [
  {
    id: "bore",
    label: "Bore",
    icon: Terminal,
    tagline: "Free, zero-config",
    color: "text-emerald-500",
    bgColor: "bg-emerald-500/10",
    borderColor: "border-emerald-500/30",
  },
  {
    id: "cloudflare",
    label: "Cloudflare",
    icon: Cloud,
    tagline: "Free or custom domain",
    color: "text-orange-500",
    bgColor: "bg-orange-500/10",
    borderColor: "border-orange-500/30",
  },
  {
    id: "ngrok",
    label: "ngrok",
    icon: Globe,
    tagline: "Free tier or paid",
    color: "text-blue-500",
    bgColor: "bg-blue-500/10",
    borderColor: "border-blue-500/30",
  },
  {
    id: "frp",
    label: "FRP",
    icon: Server,
    tagline: "Self-hosted",
    color: "text-purple-500",
    bgColor: "bg-purple-500/10",
    borderColor: "border-purple-500/30",
  },
] as const;

function getProviderMeta(id: string) {
  return PROVIDERS.find((p) => p.id === id) ?? PROVIDERS[0];
}

// =============================================================================
// Skeletons
// =============================================================================

function Skeleton({ className = "" }: { className?: string }) {
  return (
    <div
      className={`bg-ciab-bg-elevated rounded animate-pulse ${className}`}
    />
  );
}

// =============================================================================
// Main Gateway Page
// =============================================================================

export default function Gateway() {
  const [showCreateToken, setShowCreateToken] = useState(false);
  const [showExpose, setShowExpose] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showTokens, setShowTokens] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const { data: status, isLoading: statusLoading, error: statusError } = useGatewayStatus();
  const { data: tokens, isLoading: tokensLoading } = useGatewayTokens();
  const { data: tunnels, isLoading: tunnelsLoading } = useGatewayTunnels();
  const qc = useQueryClient();
  const [refreshing, setRefreshing] = useState(false);

  const isGatewayDisabled = !!statusError;

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const handleRefresh = () => {
    setRefreshing(true);
    qc.invalidateQueries({ queryKey: ["gateway-status"] });
    qc.invalidateQueries({ queryKey: ["gateway-tokens"] });
    qc.invalidateQueries({ queryKey: ["gateway-tunnels"] });
    setTimeout(() => setRefreshing(false), 600);
  };

  if (isGatewayDisabled && !statusLoading) {
    return (
      <div className="animate-fade-in">
        <GatewayOnboarding />
      </div>
    );
  }

  const activeTokenCount = tokens?.filter((t) => !t.revoked_at)?.length ?? 0;
  const activeTunnelCount = tunnels?.filter((t) => t.state === "active")?.length ?? 0;
  const activeProvider = status?.active_provider ?? "none";
  const providerMeta = getProviderMeta(activeProvider);

  return (
    <div className="space-y-5 animate-fade-in">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div className="flex items-center gap-2.5">
          <h1 className="text-xl font-semibold tracking-tight">Gateway</h1>
          {status?.enabled && (
            <span className="flex items-center gap-1.5 text-[10px] font-mono bg-state-running/10 text-state-running px-2 py-0.5 rounded-full">
              <span className="w-1.5 h-1.5 rounded-full bg-state-running animate-pulse-slow" />
              ACTIVE
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowSettings(true)}
            className="p-2 rounded-md text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
            title="Settings"
          >
            <Settings2 className="w-4 h-4" />
          </button>
          <button
            onClick={handleRefresh}
            className="p-2 rounded-md text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
            title="Refresh"
          >
            <RefreshCw className={`w-4 h-4 ${refreshing ? "animate-spin" : ""}`} />
          </button>
          <button
            onClick={() => setShowExpose(true)}
            className="btn-primary flex items-center gap-2"
          >
            <Zap className="w-4 h-4" />
            <span className="hidden sm:inline">Expose Sandbox</span>
            <span className="sm:hidden">Expose</span>
          </button>
        </div>
      </div>

      {/* Stats strip */}
      {statusLoading ? (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {[...Array(4)].map((_, i) => (
            <div key={i} className="card p-3">
              <Skeleton className="w-16 h-3 mb-2" />
              <Skeleton className="w-10 h-5" />
            </div>
          ))}
        </div>
      ) : (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          <MiniStat
            icon={providerMeta.icon}
            label="Provider"
            value={activeProvider}
            color={providerMeta.color}
          />
          <MiniStat
            icon={Link2}
            label="Tunnels"
            value={activeTunnelCount.toString()}
            color="text-ciab-steel-blue"
          />
          <button
            onClick={() => setShowTokens(!showTokens)}
            className="text-left"
          >
            <MiniStat
              icon={Key}
              label="Tokens"
              value={activeTokenCount.toString()}
              color="text-ciab-copper"
            />
          </button>
          <MiniStat
            icon={status?.lan?.enabled ? Wifi : WifiOff}
            label="LAN"
            value={status?.lan?.enabled ? "On" : "Off"}
            color={status?.lan?.enabled ? "text-state-running" : "text-ciab-text-muted"}
          />
        </div>
      )}

      {/* LAN Access Card */}
      <LanAccessCard status={status} isLoading={statusLoading} copyToClipboard={copyToClipboard} copiedId={copiedId} />

      {/* Provider Setup Banner — show if active provider not installed */}
      <ProviderSetupBanner status={status} />

      {/* Tunnels */}
      <TunnelsList
        tunnels={tunnels}
        isLoading={tunnelsLoading}
        copyToClipboard={copyToClipboard}
        copiedId={copiedId}
      />

      {/* Tokens (collapsible) */}
      <TokensSection
        tokens={tokens}
        isLoading={tokensLoading}
        isOpen={showTokens}
        onToggle={() => setShowTokens(!showTokens)}
        onCreateToken={() => setShowCreateToken(true)}
      />

      {/* Dialogs */}
      {showCreateToken && (
        <CreateTokenDialog onClose={() => setShowCreateToken(false)} />
      )}
      {showExpose && (
        <ExposeDialog onClose={() => setShowExpose(false)} />
      )}
      {showSettings && (
        <SettingsPanel
          status={status}
          onClose={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}

// =============================================================================
// Mini Stat
// =============================================================================

function MiniStat({
  icon: Icon,
  label,
  value,
  color,
}: {
  icon: any;
  label: string;
  value: string;
  color: string;
}) {
  return (
    <div className="card p-3">
      <div className="flex items-center gap-1.5 mb-1">
        <Icon className={`w-3.5 h-3.5 ${color}`} />
        <span className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">
          {label}
        </span>
      </div>
      <div className="text-base font-semibold text-ciab-text-primary capitalize">{value}</div>
    </div>
  );
}

// =============================================================================
// LAN Access Card
// =============================================================================

function LanAccessCard({
  status,
  isLoading,
  copyToClipboard,
  copiedId,
}: {
  status: any;
  isLoading: boolean;
  copyToClipboard: (text: string, id: string) => void;
  copiedId: string | null;
}) {
  const [showQr, setShowQr] = useState<string | null>(null);

  if (isLoading) return null;
  if (!status?.lan?.enabled) return null;

  const port = status.lan.advertise_port;
  const addresses: string[] = status.lan.local_addresses ?? [];
  const mdnsName = status.lan.mdns_name;

  // Build list of access URLs
  const urls: { label: string; url: string; primary?: boolean }[] = [];

  if (mdnsName) {
    urls.push({
      label: `${mdnsName}.local`,
      url: `http://${mdnsName}.local:${port}`,
      primary: true,
    });
  }

  for (const addr of addresses) {
    // Skip IPv6 link-local and loopback
    if (addr.startsWith("::") || addr === "127.0.0.1" || addr.startsWith("fe80")) continue;
    urls.push({
      label: addr,
      url: `http://${addr.includes(":") ? `[${addr}]` : addr}:${port}`,
    });
  }

  if (urls.length === 0) return null;

  const primaryUrl = urls.find((u) => u.primary)?.url ?? urls[0]?.url;

  return (
    <div className="card overflow-hidden">
      <div className="p-4">
        <div className="flex items-center gap-2 mb-3">
          <Wifi className="w-4 h-4 text-ciab-steel-blue" />
          <h3 className="text-sm font-semibold">Local Access</h3>
          <span className="flex items-center gap-1 text-[10px] font-mono bg-state-running/10 text-state-running px-1.5 py-0.5 rounded-full">
            <span className="w-1 h-1 rounded-full bg-state-running" />
            on
          </span>
          <div className="flex-1" />
          <button
            onClick={() => setShowQr(showQr ? null : primaryUrl)}
            className={`p-1.5 rounded-md transition-colors ${
              showQr ? "bg-ciab-copper/10 text-ciab-copper" : "text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover"
            }`}
            title="Show QR code"
          >
            <QrCode className="w-4 h-4" />
          </button>
        </div>

        <div className="flex gap-4">
          {/* URL list */}
          <div className="flex-1 space-y-1.5 min-w-0">
            {urls.map((entry) => (
              <div
                key={entry.url}
                className={`flex items-center gap-2 rounded-lg px-3 py-2 ${
                  entry.primary
                    ? "bg-ciab-steel-blue/5 border border-ciab-steel-blue/20"
                    : "bg-ciab-bg-primary/50"
                }`}
              >
                {entry.primary ? (
                  <Smartphone className="w-3.5 h-3.5 text-ciab-steel-blue flex-shrink-0" />
                ) : (
                  <Network className="w-3 h-3 text-ciab-text-muted/50 flex-shrink-0" />
                )}
                <code className="text-xs font-mono text-ciab-text-primary truncate flex-1">
                  {entry.url}
                </code>
                <button
                  onClick={() => copyToClipboard(entry.url, `lan-${entry.label}`)}
                  className="p-0.5 text-ciab-text-muted hover:text-ciab-text-primary transition-colors flex-shrink-0"
                  title="Copy URL"
                >
                  {copiedId === `lan-${entry.label}` ? (
                    <Check className="w-3.5 h-3.5 text-state-running" />
                  ) : (
                    <Copy className="w-3.5 h-3.5" />
                  )}
                </button>
                <button
                  onClick={() => setShowQr(showQr === entry.url ? null : entry.url)}
                  className={`p-0.5 transition-colors flex-shrink-0 ${
                    showQr === entry.url ? "text-ciab-copper" : "text-ciab-text-muted hover:text-ciab-text-primary"
                  }`}
                  title="QR Code"
                >
                  <QrCode className="w-3.5 h-3.5" />
                </button>
              </div>
            ))}
          </div>

          {/* QR Code */}
          {showQr && (
            <div className="flex-shrink-0 animate-fade-in flex flex-col items-center gap-2">
              <div className="bg-white p-2.5 rounded-lg">
                <QRCodeSVG
                  value={showQr}
                  size={120}
                  level="M"
                  bgColor="white"
                  fgColor="#1a1a1a"
                />
              </div>
              <span className="text-[9px] text-ciab-text-muted font-mono text-center max-w-[120px] truncate">
                Scan to open
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Provider Setup Banner
// =============================================================================

function ProviderSetupBanner({ status }: { status: any }) {
  const prepareProvider = usePrepareProvider();

  if (!status?.providers) return null;

  const active = status.active_provider;
  const info = status.providers.find((p: any) => p.name === active);

  // Don't show if provider is installed
  if (!info || info.installed) return null;

  const meta = getProviderMeta(active);

  return (
    <div className="card p-4 border-yellow-500/30 bg-yellow-500/5 animate-fade-in">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-3 min-w-0">
          <div className={`w-9 h-9 rounded-lg ${meta.bgColor} flex items-center justify-center flex-shrink-0`}>
            <meta.icon className={`w-4.5 h-4.5 ${meta.color}`} />
          </div>
          <div className="min-w-0">
            <div className="text-sm font-medium flex items-center gap-2">
              {meta.label} not installed
              <AlertTriangle className="w-3.5 h-3.5 text-yellow-500" />
            </div>
            <p className="text-[11px] text-ciab-text-muted">
              Install to start creating tunnels
            </p>
          </div>
        </div>
        <button
          onClick={() => prepareProvider.mutate(active)}
          disabled={prepareProvider.isPending}
          className="btn-primary text-xs px-4 py-2 flex items-center gap-2 flex-shrink-0"
        >
          {prepareProvider.isPending ? (
            <>
              <RefreshCw className="w-3.5 h-3.5 animate-spin" />
              Installing...
            </>
          ) : (
            <>
              <Download className="w-3.5 h-3.5" />
              Install
            </>
          )}
        </button>
      </div>
    </div>
  );
}

// =============================================================================
// Tunnels List
// =============================================================================

function TunnelsList({
  tunnels,
  isLoading,
  copyToClipboard,
  copiedId,
}: {
  tunnels: GatewayTunnel[] | undefined;
  isLoading: boolean;
  copyToClipboard: (text: string, id: string) => void;
  copiedId: string | null;
}) {
  const deleteTunnel = useDeleteGatewayTunnel();
  const [expandedQr, setExpandedQr] = useState<string | null>(null);

  if (isLoading) {
    return (
      <div className="space-y-2">
        <div className="flex items-center gap-2 mb-1">
          <Radio className="w-4 h-4 text-ciab-text-muted" />
          <span className="text-sm font-semibold">Public Tunnels</span>
        </div>
        {[...Array(2)].map((_, i) => (
          <div key={i} className="card p-4">
            <div className="flex items-center gap-3">
              <Skeleton className="w-10 h-10 rounded-lg flex-shrink-0" />
              <div className="flex-1 space-y-2">
                <Skeleton className="w-56 h-4" />
                <Skeleton className="w-36 h-3" />
              </div>
            </div>
          </div>
        ))}
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center gap-2 mb-2">
        <Radio className="w-4 h-4 text-ciab-text-muted" />
        <span className="text-sm font-semibold">Public Tunnels</span>
        {tunnels && tunnels.length > 0 && (
          <span className="text-[10px] font-mono bg-ciab-bg-elevated px-1.5 py-0.5 rounded-full text-ciab-text-muted">
            {tunnels.length}
          </span>
        )}
      </div>

      {!tunnels || tunnels.length === 0 ? (
        <div className="card p-6 text-center">
          <Radio className="w-8 h-8 text-ciab-text-muted/30 mx-auto mb-2" />
          <p className="text-sm text-ciab-text-muted">No public tunnels</p>
          <p className="text-[11px] text-ciab-text-muted/60 mt-0.5">
            Use "Expose Sandbox" to create a public URL with access token
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {tunnels.map((tunnel) => {
            const meta = getProviderMeta(tunnel.tunnel_type);
            const isActive = tunnel.state === "active";
            const isError = tunnel.state === "error";
            const isQrOpen = expandedQr === tunnel.id;

            return (
              <div
                key={tunnel.id}
                className={`card overflow-hidden transition-colors ${
                  isError ? "border-state-failed/30" : isActive ? `${meta.borderColor}` : ""
                }`}
              >
                <div className="p-4">
                  {/* Top row: provider + state + actions */}
                  <div className="flex items-center gap-2 mb-3">
                    <div
                      className={`w-7 h-7 rounded-md flex items-center justify-center flex-shrink-0 ${
                        isActive ? meta.bgColor : isError ? "bg-state-failed/10" : "bg-ciab-bg-elevated"
                      }`}
                    >
                      <meta.icon
                        className={`w-3.5 h-3.5 ${
                          isActive ? meta.color : isError ? "text-state-failed" : "text-ciab-text-muted"
                        }`}
                      />
                    </div>
                    <span className="text-xs font-semibold uppercase tracking-wide text-ciab-text-secondary">
                      {tunnel.tunnel_type}
                    </span>
                    <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded ${
                      isActive
                        ? "bg-state-running/10 text-state-running"
                        : isError
                        ? "bg-state-failed/10 text-state-failed"
                        : "bg-ciab-bg-elevated text-ciab-text-muted"
                    }`}>
                      {tunnel.state}
                    </span>
                    <div className="flex-1" />
                    {tunnel.sandbox_id && (
                      <span className="text-[10px] font-mono text-ciab-text-muted bg-ciab-bg-elevated px-1.5 py-0.5 rounded">
                        {truncateId(tunnel.sandbox_id)}
                      </span>
                    )}
                    <span className="text-[10px] text-ciab-text-muted/50 font-mono">
                      :{tunnel.local_port}
                    </span>
                  </div>

                  {/* URL row */}
                  <div className="flex items-center gap-2 bg-ciab-bg-primary/60 rounded-lg px-3 py-2.5 mb-2">
                    <ExternalLink className="w-3.5 h-3.5 text-ciab-text-muted/50 flex-shrink-0" />
                    <code className="text-sm font-mono text-ciab-text-primary truncate flex-1 select-all">
                      {tunnel.public_url}
                    </code>
                    <button
                      onClick={() => copyToClipboard(tunnel.public_url, tunnel.id)}
                      className="p-1 text-ciab-text-muted hover:text-ciab-text-primary transition-colors flex-shrink-0"
                      title="Copy URL"
                    >
                      {copiedId === tunnel.id ? (
                        <Check className="w-3.5 h-3.5 text-state-running" />
                      ) : (
                        <Copy className="w-3.5 h-3.5" />
                      )}
                    </button>
                    <button
                      onClick={() => setExpandedQr(isQrOpen ? null : tunnel.id)}
                      className={`p-1 transition-colors flex-shrink-0 ${
                        isQrOpen ? "text-ciab-copper" : "text-ciab-text-muted hover:text-ciab-text-primary"
                      }`}
                      title="Show QR code"
                    >
                      <QrCode className="w-3.5 h-3.5" />
                    </button>
                    <button
                      onClick={() => deleteTunnel.mutate(tunnel.id)}
                      disabled={deleteTunnel.isPending}
                      className="p-1 rounded text-ciab-text-muted hover:text-state-failed transition-colors flex-shrink-0 disabled:opacity-30"
                      title="Stop tunnel"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>

                  {/* Error message */}
                  {tunnel.error_message && (
                    <div className="flex items-start gap-1.5 text-[11px] text-state-failed mt-1">
                      <AlertTriangle className="w-3 h-3 mt-0.5 flex-shrink-0" />
                      <span>{tunnel.error_message}</span>
                    </div>
                  )}
                </div>

                {/* QR Code expandable section */}
                {isQrOpen && (
                  <div className="border-t border-ciab-border bg-ciab-bg-card/50 p-4 flex items-center justify-center gap-6 animate-fade-in">
                    <div className="bg-white p-3 rounded-xl shadow-sm">
                      <QRCodeSVG
                        value={tunnel.public_url}
                        size={140}
                        level="M"
                        bgColor="white"
                        fgColor="#1a1a1a"
                      />
                    </div>
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <Smartphone className="w-4 h-4 text-ciab-text-muted" />
                        <span className="text-xs text-ciab-text-secondary font-medium">
                          Scan with your phone
                        </span>
                      </div>
                      <p className="text-[11px] text-ciab-text-muted leading-relaxed max-w-[180px]">
                        Open your camera app and point it at this QR code to access CIAB remotely.
                      </p>
                      <code className="text-[10px] font-mono text-ciab-text-muted block truncate max-w-[180px]">
                        {tunnel.public_url}
                      </code>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// Tokens Section (collapsible)
// =============================================================================

function TokensSection({
  tokens,
  isLoading,
  isOpen,
  onToggle,
  onCreateToken,
}: {
  tokens: ClientToken[] | undefined;
  isLoading: boolean;
  isOpen: boolean;
  onToggle: () => void;
  onCreateToken: () => void;
}) {
  const revokeToken = useRevokeGatewayToken();
  const activeTokens = tokens?.filter((t) => !t.revoked_at) ?? [];

  return (
    <div>
      <button
        onClick={onToggle}
        className="flex items-center gap-2 mb-2 group"
      >
        {isOpen ? (
          <ChevronDown className="w-3.5 h-3.5 text-ciab-text-muted" />
        ) : (
          <ChevronRight className="w-3.5 h-3.5 text-ciab-text-muted" />
        )}
        <Key className="w-4 h-4 text-ciab-text-muted" />
        <span className="text-sm font-semibold">Tokens</span>
        {activeTokens.length > 0 && (
          <span className="text-[10px] font-mono bg-ciab-bg-elevated px-1.5 py-0.5 rounded-full text-ciab-text-muted">
            {activeTokens.length}
          </span>
        )}
      </button>

      {isOpen && (
        <div className="space-y-2 animate-fade-in">
          {isLoading ? (
            <div className="space-y-2">
              {[...Array(2)].map((_, i) => (
                <div key={i} className="card p-3">
                  <Skeleton className="w-full h-6" />
                </div>
              ))}
            </div>
          ) : activeTokens.length === 0 ? (
            <div className="card p-4 text-center">
              <p className="text-sm text-ciab-text-muted">No active tokens</p>
              <button
                onClick={onCreateToken}
                className="btn-primary text-xs px-3 py-1.5 mt-2 inline-flex items-center gap-1.5"
              >
                <Plus className="w-3.5 h-3.5" />
                Create Token
              </button>
            </div>
          ) : (
            <>
              <div className="flex justify-end mb-1">
                <button
                  onClick={onCreateToken}
                  className="btn-secondary text-xs px-2.5 py-1 flex items-center gap-1.5"
                >
                  <Plus className="w-3 h-3" />
                  New Token
                </button>
              </div>
              {activeTokens.map((token) => (
                <div key={token.id} className="card p-3 flex items-center gap-3">
                  <Key className="w-3.5 h-3.5 text-ciab-copper flex-shrink-0" />
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium truncate">{token.name}</div>
                    <div className="flex items-center gap-2 mt-0.5">
                      {token.scopes.map((scope, i) => (
                        <ScopeBadge key={i} scope={scope} />
                      ))}
                      <span className="text-[10px] text-ciab-text-muted font-mono">
                        {token.expires_at ? `exp ${formatRelativeTime(token.expires_at)}` : "no expiry"}
                      </span>
                    </div>
                  </div>
                  <button
                    onClick={() => revokeToken.mutate(token.id)}
                    disabled={revokeToken.isPending}
                    className="p-1 rounded text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10 transition-colors disabled:opacity-30"
                    title="Revoke"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              ))}
            </>
          )}
        </div>
      )}
    </div>
  );
}

function ScopeBadge({ scope }: { scope: TokenScope }) {
  const label = (() => {
    switch (scope.type) {
      case "full_access":
        return "Full";
      case "sandbox_access":
        return `Sandbox`;
      case "workspace_access":
        return `Workspace`;
      case "read_only":
        return "Read Only";
      case "chat_only":
        return `Chat`;
      default:
        return scope.type;
    }
  })();

  const color =
    scope.type === "full_access"
      ? "bg-ciab-copper/10 text-ciab-copper"
      : scope.type === "read_only"
      ? "bg-ciab-steel-blue/10 text-ciab-steel-blue"
      : "bg-ciab-bg-elevated text-ciab-text-secondary";

  return (
    <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded ${color}`}>
      {label}
    </span>
  );
}

// =============================================================================
// Gateway Onboarding
// =============================================================================

function GatewayOnboarding() {
  const updateConfig = useUpdateGatewayConfig();

  const handleEnable = () => {
    updateConfig.mutate({
      enabled: true,
      lan: { enabled: true, mdns_name: "ciab" },
    });
  };

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Gateway</h1>
        <p className="text-sm text-ciab-text-muted mt-0.5">
          Remote access, tunnels, and scoped tokens
        </p>
      </div>

      {/* Capabilities */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        <div className="card p-4 group hover:border-ciab-border-light transition-colors">
          <div className="w-8 h-8 rounded-lg bg-ciab-bg-elevated flex items-center justify-center mb-2.5 group-hover:bg-ciab-copper/10 transition-colors">
            <Wifi className="w-4 h-4 text-ciab-text-muted group-hover:text-ciab-copper transition-colors" />
          </div>
          <h3 className="text-sm font-medium mb-0.5">LAN Discovery</h3>
          <p className="text-[11px] text-ciab-text-muted leading-relaxed">
            Access CIAB from any device on your network
          </p>
        </div>
        <div className="card p-4 group hover:border-ciab-border-light transition-colors">
          <div className="w-8 h-8 rounded-lg bg-ciab-bg-elevated flex items-center justify-center mb-2.5 group-hover:bg-ciab-copper/10 transition-colors">
            <Radio className="w-4 h-4 text-ciab-text-muted group-hover:text-ciab-copper transition-colors" />
          </div>
          <h3 className="text-sm font-medium mb-0.5">Public Tunnels</h3>
          <p className="text-[11px] text-ciab-text-muted leading-relaxed">
            Expose via Bore, Cloudflare, ngrok, or FRP
          </p>
        </div>
        <div className="card p-4 group hover:border-ciab-border-light transition-colors">
          <div className="w-8 h-8 rounded-lg bg-ciab-bg-elevated flex items-center justify-center mb-2.5 group-hover:bg-ciab-copper/10 transition-colors">
            <Shield className="w-4 h-4 text-ciab-text-muted group-hover:text-ciab-copper transition-colors" />
          </div>
          <h3 className="text-sm font-medium mb-0.5">Scoped Tokens</h3>
          <p className="text-[11px] text-ciab-text-muted leading-relaxed">
            Fine-grained access per sandbox or workspace
          </p>
        </div>
      </div>

      {/* Enable */}
      <div className="card p-5">
        <div className="flex items-start gap-4">
          <div className="w-10 h-10 rounded-lg bg-ciab-copper/10 flex items-center justify-center flex-shrink-0">
            <Power className="w-5 h-5 text-ciab-copper" />
          </div>
          <div className="flex-1">
            <h3 className="text-sm font-semibold mb-1">Enable Gateway</h3>
            <p className="text-xs text-ciab-text-muted mb-4">
              Start using remote access, LAN discovery, and public tunnels.
            </p>
            <button
              onClick={handleEnable}
              disabled={updateConfig.isPending}
              className="btn-primary flex items-center gap-2"
            >
              {updateConfig.isPending ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  Enabling...
                </>
              ) : (
                <>
                  <Power className="w-4 h-4" />
                  Enable Gateway
                </>
              )}
            </button>

            {updateConfig.isError && (
              <div className="mt-3 p-3 bg-state-failed/5 border border-state-failed/20 rounded-lg">
                <p className="text-xs text-state-failed flex items-start gap-1.5">
                  <AlertTriangle className="w-3.5 h-3.5 mt-0.5 flex-shrink-0" />
                  <span>
                    {updateConfig.error?.message?.includes("404")
                      ? "Server needs restart with latest version. Rebuild and run: ciab server start --config config.toml"
                      : updateConfig.error?.message}
                  </span>
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Settings Panel (slide-over)
// =============================================================================

function SettingsPanel({
  status,
  onClose,
}: {
  status: any;
  onClose: () => void;
}) {
  const { data: config, isLoading: configLoading } = useGatewayConfig();
  const updateConfig = useUpdateGatewayConfig();
  const prepareProvider = usePrepareProvider();
  const [showSecrets, setShowSecrets] = useState<Record<string, boolean>>({});
  const [showAdvanced, setShowAdvanced] = useState(false);

  // Local form state
  const [lanEnabled, setLanEnabled] = useState<boolean | undefined>(undefined);
  const [mdnsName, setMdnsName] = useState("");
  const [lanPort, setLanPort] = useState("");
  const [tunnelProvider, setTunnelProvider] = useState("bore");
  const [boreServer, setBoreServer] = useState("bore.pub");
  const [boreSecret, setBoreSecret] = useState("");
  const [cfTunnelToken, setCfTunnelToken] = useState("");
  const [cfTunnelName, setCfTunnelName] = useState("");
  const [ngrokAuthtoken, setNgrokAuthtoken] = useState("");
  const [ngrokDomain, setNgrokDomain] = useState("");
  const [ngrokRegion, setNgrokRegion] = useState("");
  const [frpServerAddr, setFrpServerAddr] = useState("");
  const [frpServerPort, setFrpServerPort] = useState("");
  const [frpAuthToken, setFrpAuthToken] = useState("");
  const [frpSubdomain, setFrpSubdomain] = useState("");
  const [frpTls, setFrpTls] = useState(false);
  const [routingMode, setRoutingMode] = useState("path");
  const [baseDomain, setBaseDomain] = useState("");
  const [dnsCname, setDnsCname] = useState("");
  const [k8sIngressClass, setK8sIngressClass] = useState("");
  const [dirty, setDirty] = useState(false);

  const initialized = lanEnabled !== undefined;
  if (config && !initialized) {
    setLanEnabled(config.lan.enabled);
    setMdnsName(config.lan.mdns_name);
    setLanPort(config.lan.advertise_port.toString());
    setTunnelProvider(config.tunnel_provider ?? "bore");
    setBoreServer(config.bore?.server ?? "bore.pub");
    setBoreSecret(config.bore?.secret ?? "");
    setCfTunnelToken(config.cloudflare?.tunnel_token ?? "");
    setCfTunnelName(config.cloudflare?.tunnel_name ?? "");
    setNgrokAuthtoken(config.ngrok?.authtoken ?? "");
    setNgrokDomain(config.ngrok?.domain ?? "");
    setNgrokRegion(config.ngrok?.region ?? "");
    setFrpServerAddr(config.frp.server_addr ?? "");
    setFrpServerPort(config.frp.server_port?.toString() ?? "7000");
    setFrpAuthToken(config.frp.auth_token ?? "");
    setFrpSubdomain(config.frp.subdomain_prefix ?? "");
    setFrpTls(config.frp.tls_enable);
    setRoutingMode(config.routing.mode);
    setBaseDomain(config.routing.base_domain ?? "");
    setDnsCname(config.advanced.custom_dns_cname ?? "");
    setK8sIngressClass(config.advanced.k8s_ingress_class ?? "");
  }

  const markDirty = () => setDirty(true);
  const toggleSecret = (key: string) => setShowSecrets((s) => ({ ...s, [key]: !s[key] }));

  const providerInfo = useMemo(() => {
    if (!status?.providers) return null;
    return status.providers.find((p: any) => p.name === tunnelProvider);
  }, [status, tunnelProvider]);

  const handleSave = () => {
    const req: UpdateGatewayConfigRequest = {
      tunnel_provider: tunnelProvider,
      lan: {
        enabled: lanEnabled,
        mdns_name: mdnsName || undefined,
        advertise_port: lanPort ? parseInt(lanPort, 10) : undefined,
      },
      bore: {
        enabled: tunnelProvider === "bore",
        server: boreServer || undefined,
        secret: boreSecret || undefined,
      },
      cloudflare: {
        enabled: tunnelProvider === "cloudflare",
        tunnel_token: cfTunnelToken || undefined,
        tunnel_name: cfTunnelName || undefined,
      },
      ngrok: {
        enabled: tunnelProvider === "ngrok",
        authtoken: ngrokAuthtoken || undefined,
        domain: ngrokDomain || undefined,
        region: ngrokRegion || undefined,
      },
      frp: {
        enabled: tunnelProvider === "frp",
        server_addr: frpServerAddr || undefined,
        server_port: frpServerPort ? parseInt(frpServerPort, 10) : undefined,
        auth_token: frpAuthToken || undefined,
        subdomain_prefix: frpSubdomain || undefined,
        tls_enable: frpTls,
      },
      routing: {
        mode: routingMode,
        base_domain: baseDomain || undefined,
      },
      advanced: {
        custom_dns_cname: dnsCname || undefined,
        k8s_ingress_class: k8sIngressClass || undefined,
      },
    };

    updateConfig.mutate(req, {
      onSuccess: () => setDirty(false),
    });
  };

  const handleDisableGateway = () => {
    updateConfig.mutate({ enabled: false });
  };

  return (
    <div
      className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="absolute right-0 top-0 bottom-0 w-full max-w-lg bg-ciab-bg-primary border-l border-ciab-border shadow-2xl overflow-y-auto animate-slide-in-right"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="sticky top-0 z-10 bg-ciab-bg-primary border-b border-ciab-border px-5 py-3.5 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Settings2 className="w-4 h-4 text-ciab-copper" />
            <h2 className="text-sm font-semibold">Gateway Settings</h2>
          </div>
          <div className="flex items-center gap-2">
            {dirty && (
              <button
                onClick={handleSave}
                disabled={updateConfig.isPending}
                className="btn-primary text-xs px-3 py-1.5 flex items-center gap-1.5"
              >
                {updateConfig.isPending ? (
                  <RefreshCw className="w-3 h-3 animate-spin" />
                ) : (
                  <Check className="w-3 h-3" />
                )}
                Save
              </button>
            )}
            <button
              onClick={onClose}
              className="p-1.5 rounded text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {configLoading || !initialized ? (
          <div className="p-5 space-y-4">
            <Skeleton className="h-24" />
            <Skeleton className="h-40" />
            <Skeleton className="h-32" />
          </div>
        ) : (
          <div className="p-5 space-y-5">
            {/* LAN */}
            <section className="space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Wifi className="w-4 h-4 text-ciab-steel-blue" />
                  <h3 className="text-sm font-semibold">LAN Discovery</h3>
                </div>
                <ToggleSwitch
                  value={lanEnabled ?? true}
                  onChange={(v) => { setLanEnabled(v); markDirty(); }}
                />
              </div>
              {lanEnabled && (
                <div className="grid grid-cols-2 gap-3 animate-fade-in">
                  <div>
                    <label className="label">mDNS Name</label>
                    <input
                      type="text"
                      value={mdnsName}
                      onChange={(e) => { setMdnsName(e.target.value); markDirty(); }}
                      placeholder="ciab"
                      className="input w-full font-mono text-xs"
                    />
                  </div>
                  <div>
                    <label className="label">Port</label>
                    <input
                      type="number"
                      value={lanPort}
                      onChange={(e) => { setLanPort(e.target.value); markDirty(); }}
                      placeholder="9090"
                      className="input w-full font-mono text-xs"
                    />
                  </div>
                </div>
              )}
            </section>

            <div className="border-t border-ciab-border" />

            {/* Provider Selection */}
            <section className="space-y-3">
              <div className="flex items-center gap-2">
                <Radio className="w-4 h-4 text-ciab-copper" />
                <h3 className="text-sm font-semibold">Tunnel Provider</h3>
              </div>

              <div className="grid grid-cols-2 gap-2">
                {PROVIDERS.map((p) => {
                  const isSelected = tunnelProvider === p.id;
                  const info = status?.providers?.find((pi: any) => pi.name === p.id);
                  return (
                    <button
                      key={p.id}
                      type="button"
                      onClick={() => { setTunnelProvider(p.id); markDirty(); }}
                      className={`p-3 rounded-lg border text-left transition-all ${
                        isSelected
                          ? `${p.borderColor} ${p.bgColor} ring-1 ring-current/10`
                          : "border-ciab-border hover:border-ciab-text-muted/30"
                      }`}
                    >
                      <div className="flex items-center gap-2 mb-0.5">
                        <p.icon className={`w-3.5 h-3.5 ${isSelected ? p.color : "text-ciab-text-muted"}`} />
                        <span className="text-sm font-medium">{p.label}</span>
                      </div>
                      <div className="flex items-center gap-1.5">
                        <span className="text-[10px] text-ciab-text-muted">{p.tagline}</span>
                        {info?.installed ? (
                          <CheckCircle2 className="w-3 h-3 text-state-running" />
                        ) : (
                          <XCircle className="w-3 h-3 text-ciab-text-muted/40" />
                        )}
                      </div>
                    </button>
                  );
                })}
              </div>

              {/* Install prompt */}
              {providerInfo && !providerInfo.installed && (
                <button
                  onClick={() => prepareProvider.mutate(tunnelProvider)}
                  disabled={prepareProvider.isPending}
                  className="w-full p-2.5 rounded-lg border border-dashed border-yellow-500/40 bg-yellow-500/5 text-xs flex items-center justify-center gap-2 hover:bg-yellow-500/10 transition-colors disabled:opacity-50"
                >
                  {prepareProvider.isPending ? (
                    <>
                      <RefreshCw className="w-3.5 h-3.5 animate-spin text-yellow-500" />
                      <span className="text-yellow-600">Installing {tunnelProvider}...</span>
                    </>
                  ) : (
                    <>
                      <Download className="w-3.5 h-3.5 text-yellow-500" />
                      <span className="text-yellow-600">Install {tunnelProvider}</span>
                    </>
                  )}
                </button>
              )}

              {providerInfo?.installed && providerInfo.binary_path && (
                <div className="text-[10px] text-ciab-text-muted/60 font-mono truncate">
                  {providerInfo.binary_path}
                  {providerInfo.version && ` (${providerInfo.version})`}
                </div>
              )}
            </section>

            <div className="border-t border-ciab-border" />

            {/* Provider-specific config */}
            <section className="space-y-3">
              {tunnelProvider === "bore" && (
                <>
                  <h4 className="text-xs font-semibold text-ciab-text-secondary flex items-center gap-1.5">
                    <Terminal className="w-3.5 h-3.5" />
                    Bore Config
                  </h4>
                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <label className="label">Relay Server</label>
                      <input
                        type="text"
                        value={boreServer}
                        onChange={(e) => { setBoreServer(e.target.value); markDirty(); }}
                        placeholder="bore.pub"
                        className="input w-full font-mono text-xs"
                      />
                    </div>
                    <div>
                      <label className="label">Secret</label>
                      <SecretInput
                        value={boreSecret}
                        onChange={(v) => { setBoreSecret(v); markDirty(); }}
                        show={showSecrets.bore}
                        onToggle={() => toggleSecret("bore")}
                        placeholder="Optional"
                      />
                    </div>
                  </div>
                </>
              )}

              {tunnelProvider === "cloudflare" && (
                <>
                  <h4 className="text-xs font-semibold text-ciab-text-secondary flex items-center gap-1.5">
                    <Cloud className="w-3.5 h-3.5" />
                    Cloudflare Config
                  </h4>
                  <div className="text-[10px] text-ciab-text-muted bg-ciab-bg-elevated rounded px-2.5 py-1.5">
                    Leave token empty for free quick tunnels (*.trycloudflare.com)
                  </div>
                  <div className="space-y-3">
                    <div>
                      <label className="label">Tunnel Token</label>
                      <SecretInput
                        value={cfTunnelToken}
                        onChange={(v) => { setCfTunnelToken(v); markDirty(); }}
                        show={showSecrets.cf}
                        onToggle={() => toggleSecret("cf")}
                        placeholder="Optional — from Cloudflare dashboard"
                      />
                    </div>
                    <div>
                      <label className="label">Tunnel Name</label>
                      <input
                        type="text"
                        value={cfTunnelName}
                        onChange={(e) => { setCfTunnelName(e.target.value); markDirty(); }}
                        placeholder="my-ciab-tunnel"
                        className="input w-full font-mono text-xs"
                      />
                    </div>
                  </div>
                </>
              )}

              {tunnelProvider === "ngrok" && (
                <>
                  <h4 className="text-xs font-semibold text-ciab-text-secondary flex items-center gap-1.5">
                    <Globe className="w-3.5 h-3.5" />
                    ngrok Config
                  </h4>
                  <div className="space-y-3">
                    <div>
                      <label className="label">Authtoken</label>
                      <SecretInput
                        value={ngrokAuthtoken}
                        onChange={(v) => { setNgrokAuthtoken(v); markDirty(); }}
                        show={showSecrets.ngrok}
                        onToggle={() => toggleSecret("ngrok")}
                        placeholder="From dashboard.ngrok.com"
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <label className="label">Domain</label>
                        <input
                          type="text"
                          value={ngrokDomain}
                          onChange={(e) => { setNgrokDomain(e.target.value); markDirty(); }}
                          placeholder="Optional"
                          className="input w-full font-mono text-xs"
                        />
                      </div>
                      <div>
                        <label className="label">Region</label>
                        <select
                          value={ngrokRegion}
                          onChange={(e) => { setNgrokRegion(e.target.value); markDirty(); }}
                          className="input w-full text-xs"
                        >
                          <option value="">Auto</option>
                          <option value="us">US</option>
                          <option value="eu">EU</option>
                          <option value="ap">Asia Pacific</option>
                          <option value="au">Australia</option>
                          <option value="sa">South America</option>
                          <option value="jp">Japan</option>
                          <option value="in">India</option>
                        </select>
                      </div>
                    </div>
                  </div>
                </>
              )}

              {tunnelProvider === "frp" && (
                <>
                  <h4 className="text-xs font-semibold text-ciab-text-secondary flex items-center gap-1.5">
                    <Server className="w-3.5 h-3.5" />
                    FRP Config
                  </h4>
                  <div className="space-y-3">
                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <label className="label">Server</label>
                        <input
                          type="text"
                          value={frpServerAddr}
                          onChange={(e) => { setFrpServerAddr(e.target.value); markDirty(); }}
                          placeholder="frp.example.com"
                          className="input w-full font-mono text-xs"
                        />
                      </div>
                      <div>
                        <label className="label">Port</label>
                        <input
                          type="number"
                          value={frpServerPort}
                          onChange={(e) => { setFrpServerPort(e.target.value); markDirty(); }}
                          placeholder="7000"
                          className="input w-full font-mono text-xs"
                        />
                      </div>
                    </div>
                    <div>
                      <label className="label">Auth Token</label>
                      <SecretInput
                        value={frpAuthToken}
                        onChange={(v) => { setFrpAuthToken(v); markDirty(); }}
                        show={showSecrets.frp}
                        onToggle={() => toggleSecret("frp")}
                        placeholder="FRP server token"
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <label className="label">Subdomain Prefix</label>
                        <input
                          type="text"
                          value={frpSubdomain}
                          onChange={(e) => { setFrpSubdomain(e.target.value); markDirty(); }}
                          placeholder="ciab"
                          className="input w-full font-mono text-xs"
                        />
                      </div>
                      <div className="flex items-end pb-0.5">
                        <label className="flex items-center gap-2 cursor-pointer">
                          <ToggleSwitch
                            value={frpTls}
                            onChange={(v) => { setFrpTls(v); markDirty(); }}
                          />
                          <span className="text-xs">TLS</span>
                        </label>
                      </div>
                    </div>
                  </div>
                </>
              )}
            </section>

            <div className="border-t border-ciab-border" />

            {/* Routing */}
            <section className="space-y-3">
              <div className="flex items-center gap-2">
                <Network className="w-3.5 h-3.5 text-ciab-text-muted" />
                <h4 className="text-xs font-semibold text-ciab-text-secondary">Routing</h4>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="label">Mode</label>
                  <select
                    value={routingMode}
                    onChange={(e) => { setRoutingMode(e.target.value); markDirty(); }}
                    className="input w-full text-xs"
                  >
                    <option value="path">Path-based</option>
                    <option value="subdomain">Subdomain</option>
                  </select>
                </div>
                <div>
                  <label className="label">Base Domain</label>
                  <input
                    type="text"
                    value={baseDomain}
                    onChange={(e) => { setBaseDomain(e.target.value); markDirty(); }}
                    placeholder="Optional"
                    className="input w-full font-mono text-xs"
                  />
                </div>
              </div>
            </section>

            {/* Advanced (collapsed) */}
            <div>
              <button
                onClick={() => setShowAdvanced(!showAdvanced)}
                className="flex items-center gap-1.5 text-[11px] text-ciab-text-muted hover:text-ciab-text-secondary transition-colors"
              >
                {showAdvanced ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
                Advanced
              </button>
              {showAdvanced && (
                <div className="mt-3 space-y-3 animate-fade-in">
                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <label className="label">DNS CNAME</label>
                      <input
                        type="text"
                        value={dnsCname}
                        onChange={(e) => { setDnsCname(e.target.value); markDirty(); }}
                        placeholder="gateway.example.com"
                        className="input w-full font-mono text-xs"
                      />
                    </div>
                    <div>
                      <label className="label">K8s Ingress Class</label>
                      <input
                        type="text"
                        value={k8sIngressClass}
                        onChange={(e) => { setK8sIngressClass(e.target.value); markDirty(); }}
                        placeholder="nginx"
                        className="input w-full font-mono text-xs"
                      />
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* Disable */}
            <div className="border-t border-ciab-border pt-3">
              <button
                onClick={handleDisableGateway}
                disabled={updateConfig.isPending}
                className="text-xs text-state-failed/70 hover:text-state-failed transition-colors flex items-center gap-1.5"
              >
                <Power className="w-3 h-3" />
                Disable Gateway
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// Shared Components
// =============================================================================

function ToggleSwitch({
  value,
  onChange,
}: {
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={value}
      onClick={() => onChange(!value)}
      className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors flex-shrink-0 ${
        value ? "bg-ciab-copper" : "bg-ciab-bg-elevated border border-ciab-border"
      }`}
    >
      <span
        className={`inline-block h-3.5 w-3.5 rounded-full bg-white shadow-sm transition-transform duration-150 ${
          value ? "translate-x-[18px]" : "translate-x-[3px]"
        }`}
      />
    </button>
  );
}

function SecretInput({
  value,
  onChange,
  show,
  onToggle,
  placeholder,
}: {
  value: string;
  onChange: (v: string) => void;
  show?: boolean;
  onToggle: () => void;
  placeholder?: string;
}) {
  return (
    <div className="relative">
      <input
        type={show ? "text" : "password"}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="input w-full font-mono text-xs pr-8"
      />
      <button
        type="button"
        onClick={onToggle}
        className="absolute right-2 top-1/2 -translate-y-1/2 text-ciab-text-muted hover:text-ciab-text-primary transition-colors"
      >
        {show ? <Eye className="w-3.5 h-3.5" /> : <EyeOff className="w-3.5 h-3.5" />}
      </button>
    </div>
  );
}

// =============================================================================
// Sandbox Picker
// =============================================================================

function SandboxPicker({
  value,
  onChange,
  label,
}: {
  value: string;
  onChange: (id: string) => void;
  label?: string;
}) {
  const { data: sandboxList, isLoading } = useSandboxes();
  const [search, setSearch] = useState("");
  const [open, setOpen] = useState(false);

  const filtered = useMemo(() => {
    if (!sandboxList) return [];
    const q = search.toLowerCase();
    return sandboxList.filter(
      (s) =>
        (s.name?.toLowerCase().includes(q) ?? false) ||
        s.id.toLowerCase().includes(q) ||
        s.agent_provider.toLowerCase().includes(q)
    );
  }, [sandboxList, search]);

  const selected = sandboxList?.find((s) => s.id === value);

  return (
    <div className="relative">
      <label className="label">{label ?? "Sandbox"}</label>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="input w-full text-left flex items-center justify-between"
      >
        {selected ? (
          <div className="flex items-center gap-2 min-w-0">
            <AgentProviderIcon provider={selected.agent_provider} size={14} />
            <span className="text-sm truncate">
              {selected.name ?? truncateId(selected.id)}
            </span>
            <SandboxStateBadge state={selected.state} />
          </div>
        ) : (
          <span className="text-ciab-text-muted text-sm">
            {isLoading ? "Loading..." : "Select sandbox"}
          </span>
        )}
        <ChevronDown className="w-3.5 h-3.5 text-ciab-text-muted flex-shrink-0" />
      </button>

      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
          <div className="absolute top-full left-0 right-0 mt-1 bg-ciab-bg-card border border-ciab-border rounded-lg shadow-lg z-50 animate-slide-down max-h-64 flex flex-col">
            <div className="p-2 border-b border-ciab-border/50">
              <div className="relative">
                <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-ciab-text-muted" />
                <input
                  type="text"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  placeholder="Search..."
                  className="input w-full pl-7 py-1.5 text-xs"
                  autoFocus
                />
              </div>
            </div>
            <div className="overflow-y-auto">
              {isLoading ? (
                <div className="p-3 space-y-2">
                  {[...Array(3)].map((_, i) => <Skeleton key={i} className="h-8" />)}
                </div>
              ) : filtered.length === 0 ? (
                <div className="p-4 text-center text-xs text-ciab-text-muted">
                  {search ? "No matches" : "No sandboxes"}
                </div>
              ) : (
                filtered.map((sb) => (
                  <button
                    key={sb.id}
                    onClick={() => { onChange(sb.id); setOpen(false); setSearch(""); }}
                    className={`w-full px-3 py-2 flex items-center gap-2.5 text-left hover:bg-ciab-bg-hover transition-colors ${
                      sb.id === value ? "bg-ciab-bg-hover/50" : ""
                    }`}
                  >
                    <AgentProviderIcon provider={sb.agent_provider} size={14} />
                    <div className="flex-1 min-w-0">
                      <div className="text-sm truncate">{sb.name ?? truncateId(sb.id)}</div>
                      <div className="text-[10px] font-mono text-ciab-text-muted truncate">
                        {truncateId(sb.id)} · {sb.agent_provider}
                      </div>
                    </div>
                    <SandboxStateBadge state={sb.state} />
                  </button>
                ))
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

// =============================================================================
// Workspace Picker
// =============================================================================

function WorkspacePicker({
  value,
  onChange,
}: {
  value: string;
  onChange: (id: string) => void;
}) {
  const { data: workspaceList, isLoading } = useWorkspaces();
  const [search, setSearch] = useState("");
  const [open, setOpen] = useState(false);

  const filtered = useMemo(() => {
    if (!workspaceList) return [];
    const q = search.toLowerCase();
    return workspaceList.filter(
      (w) => w.name.toLowerCase().includes(q) || w.id.toLowerCase().includes(q)
    );
  }, [workspaceList, search]);

  const selected = workspaceList?.find((w) => w.id === value);

  return (
    <div className="relative">
      <label className="label">Workspace</label>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="input w-full text-left flex items-center justify-between"
      >
        {selected ? (
          <span className="text-sm truncate">{selected.name}</span>
        ) : (
          <span className="text-ciab-text-muted text-sm">
            {isLoading ? "Loading..." : "Select workspace"}
          </span>
        )}
        <ChevronDown className="w-3.5 h-3.5 text-ciab-text-muted flex-shrink-0" />
      </button>

      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
          <div className="absolute top-full left-0 right-0 mt-1 bg-ciab-bg-card border border-ciab-border rounded-lg shadow-lg z-50 animate-slide-down max-h-64 flex flex-col">
            <div className="p-2 border-b border-ciab-border/50">
              <div className="relative">
                <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-ciab-text-muted" />
                <input
                  type="text"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  placeholder="Search..."
                  className="input w-full pl-7 py-1.5 text-xs"
                  autoFocus
                />
              </div>
            </div>
            <div className="overflow-y-auto">
              {isLoading ? (
                <div className="p-3 space-y-2">
                  {[...Array(3)].map((_, i) => <Skeleton key={i} className="h-8" />)}
                </div>
              ) : filtered.length === 0 ? (
                <div className="p-4 text-center text-xs text-ciab-text-muted">
                  {search ? "No matches" : "No workspaces"}
                </div>
              ) : (
                filtered.map((ws) => (
                  <button
                    key={ws.id}
                    onClick={() => { onChange(ws.id); setOpen(false); setSearch(""); }}
                    className={`w-full px-3 py-2 text-left hover:bg-ciab-bg-hover transition-colors ${
                      ws.id === value ? "bg-ciab-bg-hover/50" : ""
                    }`}
                  >
                    <div className="text-sm">{ws.name}</div>
                    <div className="text-[10px] font-mono text-ciab-text-muted">{truncateId(ws.id)}</div>
                  </button>
                ))
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

// =============================================================================
// Create Token Dialog
// =============================================================================

function CreateTokenDialog({ onClose }: { onClose: () => void }) {
  const [name, setName] = useState("");
  const [scopeType, setScopeType] = useState<TokenScopeType>("full_access");
  const [scopeId, setScopeId] = useState("");
  const [expiresSecs, setExpiresSecs] = useState("");
  const [createdToken, setCreatedToken] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const createToken = useCreateGatewayToken();

  const handleCreate = () => {
    const scope: TokenScope = { type: scopeType };
    if (scopeType === "sandbox_access" || scopeType === "chat_only") {
      scope.sandbox_id = scopeId;
    } else if (scopeType === "workspace_access") {
      scope.workspace_id = scopeId;
    }

    const request: CreateTokenRequest = {
      name,
      scopes: [scope],
      expires_secs: expiresSecs ? parseInt(expiresSecs, 10) : undefined,
    };

    createToken.mutate(request, {
      onSuccess: (data) => setCreatedToken(data.token),
    });
  };

  const copyToken = () => {
    if (createdToken) {
      navigator.clipboard.writeText(createdToken);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const needsSandboxId = scopeType === "sandbox_access" || scopeType === "chat_only";
  const needsWorkspaceId = scopeType === "workspace_access";
  const needsId = needsSandboxId || needsWorkspaceId;
  const isValid = name.trim() && (!needsId || scopeId.trim());

  const scopeOptions: { value: TokenScopeType; label: string; icon: typeof Key }[] = [
    { value: "full_access", label: "Full Access", icon: Shield },
    { value: "sandbox_access", label: "Sandbox", icon: Server },
    { value: "workspace_access", label: "Workspace", icon: Layers },
    { value: "read_only", label: "Read Only", icon: Eye },
    { value: "chat_only", label: "Chat Only", icon: Terminal },
  ];

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-end sm:items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-t-xl sm:rounded-lg w-full max-sm:max-h-[90vh] sm:max-w-md animate-scale-in flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between p-4 border-b border-ciab-border flex-shrink-0">
          <div className="flex items-center gap-2">
            <Key className="w-4 h-4 text-ciab-copper" />
            <h2 className="text-sm font-semibold">
              {createdToken ? "Token Created" : "Create Token"}
            </h2>
          </div>
          <button onClick={onClose} className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors p-0.5">
            <X className="w-4 h-4" />
          </button>
        </div>

        {createdToken ? (
          <div className="p-4 space-y-4 animate-fade-in">
            <div className="bg-ciab-copper/5 border border-ciab-copper/20 rounded-lg p-3">
              <div className="flex items-center gap-2 mb-2">
                <Shield className="w-4 h-4 text-ciab-copper" />
                <span className="text-xs font-medium text-ciab-copper">
                  Save this token — shown once only
                </span>
              </div>
              <div className="flex items-center gap-2">
                <code className="flex-1 text-xs font-mono text-ciab-text-primary bg-ciab-bg-primary px-2 py-1.5 rounded break-all select-all">
                  {createdToken}
                </code>
                <button onClick={copyToken} className="btn-secondary p-1.5 flex-shrink-0">
                  {copied ? <Check className="w-4 h-4 text-state-running" /> : <Copy className="w-4 h-4" />}
                </button>
              </div>
            </div>
            <div className="flex justify-end">
              <button onClick={onClose} className="btn-primary text-sm px-4 py-1.5">Done</button>
            </div>
          </div>
        ) : (
          <>
            <div className="p-4 space-y-4 max-h-[60vh] overflow-y-auto">
              <div>
                <label className="label">Name</label>
                <input
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g. mobile-access, ci-deploy"
                  className="input w-full"
                />
              </div>

              <div>
                <label className="label">Scope</label>
                <div className="flex flex-wrap gap-1.5">
                  {scopeOptions.map((opt) => (
                    <button
                      key={opt.value}
                      type="button"
                      onClick={() => { setScopeType(opt.value); setScopeId(""); }}
                      className={`px-2.5 py-1.5 rounded-md border text-xs flex items-center gap-1.5 transition-all ${
                        scopeType === opt.value
                          ? "border-ciab-copper bg-ciab-copper/5 text-ciab-copper"
                          : "border-ciab-border text-ciab-text-muted hover:border-ciab-border-light"
                      }`}
                    >
                      <opt.icon className="w-3 h-3" />
                      {opt.label}
                    </button>
                  ))}
                </div>
              </div>

              {needsSandboxId && <SandboxPicker value={scopeId} onChange={setScopeId} />}
              {needsWorkspaceId && <WorkspacePicker value={scopeId} onChange={setScopeId} />}

              <div>
                <label className="label">Expiry</label>
                <div className="flex gap-1.5">
                  {[
                    { label: "1h", value: "3600" },
                    { label: "24h", value: "86400" },
                    { label: "7d", value: "604800" },
                    { label: "30d", value: "2592000" },
                    { label: "Never", value: "" },
                  ].map((p) => (
                    <button
                      key={p.label}
                      type="button"
                      onClick={() => setExpiresSecs(p.value)}
                      className={`px-2 py-1 text-xs font-mono rounded transition-colors ${
                        expiresSecs === p.value
                          ? "bg-ciab-copper/10 text-ciab-copper border border-ciab-copper/30"
                          : "bg-ciab-bg-elevated text-ciab-text-muted hover:text-ciab-text-primary border border-transparent"
                      }`}
                    >
                      {p.label}
                    </button>
                  ))}
                </div>
              </div>
            </div>

            <div className="flex justify-end gap-2 p-4 border-t border-ciab-border">
              <button onClick={onClose} className="btn-secondary text-sm px-3 py-1.5">Cancel</button>
              <button
                onClick={handleCreate}
                disabled={!isValid || createToken.isPending}
                className="btn-primary disabled:opacity-30 text-sm px-3 py-1.5 flex items-center gap-2"
              >
                {createToken.isPending ? (
                  <>
                    <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                    Creating...
                  </>
                ) : (
                  <>
                    <Key className="w-3.5 h-3.5" />
                    Create
                  </>
                )}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// Expose Dialog
// =============================================================================

function ExposeDialog({ onClose }: { onClose: () => void }) {
  const [sandboxId, setSandboxId] = useState("");
  const [tokenName, setTokenName] = useState("");
  const [expiresSecs, setExpiresSecs] = useState("");
  const [showOptions, setShowOptions] = useState(false);
  const [result, setResult] = useState<{ token: string; publicUrl: string } | null>(null);
  const [copied, setCopied] = useState<string | null>(null);

  const expose = useExposeGateway();

  const handleExpose = () => {
    const request: ExposeRequest = {
      sandbox_id: sandboxId,
      token_name: tokenName || undefined,
      expires_secs: expiresSecs ? parseInt(expiresSecs, 10) : undefined,
    };

    expose.mutate(request, {
      onSuccess: (data) => {
        setResult({ token: data.token, publicUrl: data.tunnel.public_url });
      },
    });
  };

  const copy = (text: string, key: string) => {
    navigator.clipboard.writeText(text);
    setCopied(key);
    setTimeout(() => setCopied(null), 2000);
  };

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-end sm:items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-t-xl sm:rounded-lg w-full max-sm:max-h-[90vh] sm:max-w-lg animate-scale-in flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between p-4 border-b border-ciab-border flex-shrink-0">
          <div className="flex items-center gap-2">
            <Zap className="w-4 h-4 text-ciab-copper" />
            <h2 className="text-sm font-semibold">
              {result ? "Sandbox Exposed" : "Expose Sandbox"}
            </h2>
          </div>
          <button onClick={onClose} className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors p-0.5">
            <X className="w-4 h-4" />
          </button>
        </div>

        {result ? (
          <div className="p-4 space-y-4 animate-fade-in">
            {/* QR + URL */}
            <div className="flex items-start gap-4">
              <div className="bg-white p-2 rounded-lg flex-shrink-0">
                <QRCodeSVG
                  value={result.publicUrl}
                  size={100}
                  level="M"
                  bgColor="white"
                  fgColor="#1a1a1a"
                />
              </div>
              <div className="flex-1 min-w-0 space-y-2">
                <div>
                  <label className="label">Public URL</label>
                  <div className="flex items-center gap-1.5">
                    <code className="flex-1 text-xs font-mono text-ciab-text-primary bg-ciab-bg-primary px-2 py-1.5 rounded select-all truncate">
                      {result.publicUrl}
                    </code>
                    <button onClick={() => copy(result.publicUrl, "url")} className="btn-secondary p-1 flex-shrink-0">
                      {copied === "url" ? <Check className="w-3.5 h-3.5 text-state-running" /> : <Copy className="w-3.5 h-3.5" />}
                    </button>
                  </div>
                </div>
                <p className="text-[10px] text-ciab-text-muted flex items-center gap-1">
                  <Smartphone className="w-3 h-3" />
                  Scan QR with your phone to access
                </p>
              </div>
            </div>

            {/* Token */}
            <div className="bg-ciab-copper/5 border border-ciab-copper/20 rounded-lg p-3">
              <div className="flex items-center gap-2 mb-2">
                <Shield className="w-4 h-4 text-ciab-copper" />
                <span className="text-xs font-medium text-ciab-copper">Access Token (save — shown once)</span>
              </div>
              <div className="flex items-center gap-2">
                <code className="flex-1 text-xs font-mono text-ciab-text-primary bg-ciab-bg-primary px-2 py-1.5 rounded break-all select-all">
                  {result.token}
                </code>
                <button onClick={() => copy(result.token, "token")} className="btn-secondary p-1.5 flex-shrink-0">
                  {copied === "token" ? <Check className="w-4 h-4 text-state-running" /> : <Copy className="w-4 h-4" />}
                </button>
              </div>
            </div>

            <div className="flex justify-end">
              <button onClick={onClose} className="btn-primary text-sm px-4 py-1.5">Done</button>
            </div>
          </div>
        ) : (
          <>
            <div className="p-4 space-y-4">
              <SandboxPicker value={sandboxId} onChange={setSandboxId} label="Sandbox" />

              <button
                type="button"
                onClick={() => setShowOptions(!showOptions)}
                className="flex items-center gap-1.5 text-xs text-ciab-text-muted hover:text-ciab-text-secondary transition-colors"
              >
                {showOptions ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
                Options
              </button>

              {showOptions && (
                <div className="space-y-3 animate-fade-in">
                  <div>
                    <label className="label">Token Name</label>
                    <input
                      type="text"
                      value={tokenName}
                      onChange={(e) => setTokenName(e.target.value)}
                      placeholder="Auto-generated if empty"
                      className="input w-full"
                    />
                  </div>
                  <div>
                    <label className="label">Expiry</label>
                    <div className="flex gap-1.5">
                      {[
                        { label: "1h", value: "3600" },
                        { label: "24h", value: "86400" },
                        { label: "7d", value: "604800" },
                        { label: "Never", value: "" },
                      ].map((p) => (
                        <button
                          key={p.label}
                          type="button"
                          onClick={() => setExpiresSecs(p.value)}
                          className={`px-2 py-1 text-xs font-mono rounded transition-colors ${
                            expiresSecs === p.value
                              ? "bg-ciab-copper/10 text-ciab-copper border border-ciab-copper/30"
                              : "bg-ciab-bg-elevated text-ciab-text-muted hover:text-ciab-text-primary border border-transparent"
                          }`}
                        >
                          {p.label}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
              )}
            </div>

            <div className="flex justify-end gap-2 p-4 border-t border-ciab-border">
              <button onClick={onClose} className="btn-secondary text-sm px-3 py-1.5">Cancel</button>
              <button
                onClick={handleExpose}
                disabled={!sandboxId.trim() || expose.isPending}
                className="btn-primary disabled:opacity-30 text-sm px-3 py-1.5 flex items-center gap-2"
              >
                {expose.isPending ? (
                  <>
                    <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                    Exposing...
                  </>
                ) : (
                  <>
                    <Zap className="w-3.5 h-3.5" />
                    Expose
                  </>
                )}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
