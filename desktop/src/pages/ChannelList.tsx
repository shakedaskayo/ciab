import { useState } from "react";
import { Link } from "react-router";
import {
  Plus,
  Trash2,
  Play,
  Square,
  RefreshCw,
  Search,
  X,
  Zap,
  Link2,
  Layers,
  Copy,
  Check,
} from "lucide-react";
import {
  useChannels,
  useDeleteChannel,
  useStartChannel,
  useStopChannel,
} from "@/lib/hooks/use-channels";
import { formatRelativeTime, truncateId } from "@/lib/utils/format";
import LoadingSpinner from "@/components/shared/LoadingSpinner";
import CreateChannelDialog from "@/features/channel/CreateChannelDialog";
import type { Channel, ChannelProvider, ChannelState } from "@/lib/api/types";
import { useQueryClient } from "@tanstack/react-query";

// =============================================================================
// Real vendor SVG icons
// =============================================================================

export function WhatsAppIcon({ size = 20 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347m-5.421 7.403h-.004a9.87 9.87 0 01-5.031-1.378l-.361-.214-3.741.982.998-3.648-.235-.374a9.86 9.86 0 01-1.51-5.26c.001-5.45 4.436-9.884 9.888-9.884 2.64 0 5.122 1.03 6.988 2.898a9.825 9.825 0 012.893 6.994c-.003 5.45-4.437 9.884-9.885 9.884m8.413-18.297A11.815 11.815 0 0012.05 0C5.495 0 .16 5.335.157 11.892c0 2.096.547 4.142 1.588 5.945L.057 24l6.305-1.654a11.882 11.882 0 005.683 1.448h.005c6.554 0 11.89-5.335 11.893-11.893a11.821 11.821 0 00-3.48-8.413z"
        fill="#25D366"
      />
    </svg>
  );
}

export function SlackIcon({ size = 20 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
      <path d="M5.042 15.165a2.528 2.528 0 0 1-2.52 2.523A2.528 2.528 0 0 1 0 15.165a2.527 2.527 0 0 1 2.522-2.52h2.52v2.52zm1.271 0a2.527 2.527 0 0 1 2.521-2.52 2.527 2.527 0 0 1 2.521 2.52v6.313A2.528 2.528 0 0 1 8.834 24a2.528 2.528 0 0 1-2.521-2.522v-6.313z" fill="#E01E5A"/>
      <path d="M8.834 5.042a2.528 2.528 0 0 1-2.521-2.52A2.528 2.528 0 0 1 8.834 0a2.528 2.528 0 0 1 2.521 2.522v2.52H8.834zm0 1.271a2.528 2.528 0 0 1 2.521 2.521 2.528 2.528 0 0 1-2.521 2.521H2.522A2.528 2.528 0 0 1 0 8.834a2.528 2.528 0 0 1 2.522-2.521h6.312z" fill="#36C5F0"/>
      <path d="M18.956 8.834a2.528 2.528 0 0 1 2.522-2.521A2.528 2.528 0 0 1 24 8.834a2.528 2.528 0 0 1-2.522 2.521h-2.522V8.834zm-1.27 0a2.528 2.528 0 0 1-2.523 2.521 2.527 2.527 0 0 1-2.52-2.521V2.522A2.527 2.527 0 0 1 15.163 0a2.528 2.528 0 0 1 2.523 2.522v6.312z" fill="#2EB67D"/>
      <path d="M15.163 18.956a2.528 2.528 0 0 1 2.523 2.522A2.528 2.528 0 0 1 15.163 24a2.527 2.527 0 0 1-2.52-2.522v-2.522h2.52zm0-1.27a2.527 2.527 0 0 1-2.52-2.523 2.527 2.527 0 0 1 2.52-2.52h6.315A2.528 2.528 0 0 1 24 15.163a2.528 2.528 0 0 1-2.522 2.523h-6.315z" fill="#ECB22E"/>
    </svg>
  );
}

export function WebhookIcon({ size = 20 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="5" cy="17" r="3" stroke="#C4693D" strokeWidth="1.5" fill="none" />
      <circle cx="19" cy="17" r="3" stroke="#C4693D" strokeWidth="1.5" fill="none" />
      <circle cx="12" cy="5" r="3" stroke="#C4693D" strokeWidth="1.5" fill="none" />
      <path d="M12 8v3.5c0 1.38-1.12 2.5-2.5 2.5H6.5" stroke="#C4693D" strokeWidth="1.5" strokeLinecap="round" />
      <path d="M8 17h4.5c1.38 0 2.5-1.12 2.5-2.5V11" stroke="#C4693D" strokeWidth="1.5" strokeLinecap="round" />
      <path d="M15.5 14c1.38 0 2.5 1.12 2.5 2.5V17" stroke="#C4693D" strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  );
}

export function ChannelProviderIcon({ provider, size = 20 }: { provider: ChannelProvider; size?: number }) {
  switch (provider) {
    case "whatsapp": return <WhatsAppIcon size={size} />;
    case "slack": return <SlackIcon size={size} />;
    case "webhook": return <WebhookIcon size={size} />;
  }
}

// =============================================================================
// State styles
// =============================================================================

const PROVIDER_LABELS: Record<ChannelProvider, string> = {
  whatsapp: "WhatsApp",
  slack: "Slack",
  webhook: "Webhook",
};

const PROVIDER_COLORS: Record<ChannelProvider, { gradient: string; border: string; accent: string }> = {
  whatsapp: { gradient: "from-[#25D366]/15 to-[#25D366]/5", border: "border-[#25D366]/20", accent: "#25D366" },
  slack: { gradient: "from-[#611f69]/15 to-[#611f69]/5", border: "border-[#611f69]/20", accent: "#611f69" },
  webhook: { gradient: "from-ciab-copper/15 to-ciab-copper/5", border: "border-ciab-copper/20", accent: "#C4693D" },
};

const STATE_STYLES: Record<ChannelState, { bg: string; text: string; label: string; dot?: string }> = {
  inactive: { bg: "bg-ciab-bg-elevated", text: "text-ciab-text-muted", label: "Inactive" },
  pairing: { bg: "bg-amber-500/10", text: "text-amber-500", label: "Pairing", dot: "animate-pulse" },
  connected: { bg: "bg-emerald-500/10", text: "text-emerald-500", label: "Connected", dot: "animate-pulse-slow" },
  reconnecting: { bg: "bg-amber-500/10", text: "text-amber-500", label: "Reconnecting", dot: "animate-pulse" },
  failed: { bg: "bg-state-failed/10", text: "text-state-failed", label: "Failed" },
  stopped: { bg: "bg-ciab-bg-elevated", text: "text-ciab-text-muted", label: "Stopped" },
};

// =============================================================================
// Empty State Illustration
// =============================================================================

function ChannelsEmptyIllustration() {
  return (
    <svg width="280" height="160" viewBox="0 0 280 160" fill="none" xmlns="http://www.w3.org/2000/svg">
      {/* Background glow */}
      <defs>
        <radialGradient id="ch-glow" cx="140" cy="80" r="100" gradientUnits="userSpaceOnUse">
          <stop stopColor="#C4693D" stopOpacity="0.08" />
          <stop offset="1" stopColor="#C4693D" stopOpacity="0" />
        </radialGradient>
        <linearGradient id="ch-flow" x1="0" y1="80" x2="280" y2="80" gradientUnits="userSpaceOnUse">
          <stop stopColor="#C4693D" stopOpacity="0" />
          <stop offset="0.3" stopColor="#C4693D" stopOpacity="0.4" />
          <stop offset="0.7" stopColor="#C4693D" stopOpacity="0.4" />
          <stop offset="1" stopColor="#C4693D" stopOpacity="0" />
        </linearGradient>
      </defs>
      <rect width="280" height="160" fill="url(#ch-glow)" />

      {/* Flow lines */}
      <path d="M50 70 L100 70 Q110 70 110 80 L110 80 Q110 90 120 90 L130 90" stroke="url(#ch-flow)" strokeWidth="1" strokeDasharray="4 3" opacity="0.5" />
      <path d="M50 90 L100 90 Q110 90 110 80 L110 80 Q110 70 120 70 L130 70" stroke="url(#ch-flow)" strokeWidth="1" strokeDasharray="4 3" opacity="0.5" />
      <path d="M50 80 L130 80" stroke="url(#ch-flow)" strokeWidth="1.5" strokeDasharray="4 3" opacity="0.6" />
      <path d="M150 80 L220 80" stroke="url(#ch-flow)" strokeWidth="1.5" strokeDasharray="4 3" opacity="0.6" />
      <path d="M150 70 L160 70 Q170 70 170 80 L170 80 Q170 90 180 90 L220 90" stroke="url(#ch-flow)" strokeWidth="1" strokeDasharray="4 3" opacity="0.5" />
      <path d="M150 90 L160 90 Q170 90 170 80 L170 80 Q170 70 180 70 L220 70" stroke="url(#ch-flow)" strokeWidth="1" strokeDasharray="4 3" opacity="0.5" />

      {/* WhatsApp icon (left) */}
      <rect x="18" y="58" width="30" height="30" rx="8" fill="#111114" stroke="#25D366" strokeWidth="1" opacity="0.9" />
      <g transform="translate(25, 65) scale(0.67)">
        <path d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347m-5.421 7.403h-.004a9.87 9.87 0 01-5.031-1.378l-.361-.214-3.741.982.998-3.648-.235-.374a9.86 9.86 0 01-1.51-5.26c.001-5.45 4.436-9.884 9.888-9.884 2.64 0 5.122 1.03 6.988 2.898a9.825 9.825 0 012.893 6.994c-.003 5.45-4.437 9.884-9.885 9.884m8.413-18.297A11.815 11.815 0 0012.05 0C5.495 0 .16 5.335.157 11.892c0 2.096.547 4.142 1.588 5.945L.057 24l6.305-1.654a11.882 11.882 0 005.683 1.448h.005c6.554 0 11.89-5.335 11.893-11.893a11.821 11.821 0 00-3.48-8.413z" fill="#25D366"/>
      </g>

      {/* Slack icon (left) */}
      <rect x="18" y="95" width="30" height="30" rx="8" fill="#111114" stroke="#611f69" strokeWidth="1" opacity="0.9" />
      <g transform="translate(27, 103) scale(0.55)">
        <path d="M5.042 15.165a2.528 2.528 0 0 1-2.52 2.523A2.528 2.528 0 0 1 0 15.165a2.527 2.527 0 0 1 2.522-2.52h2.52v2.52zm1.271 0a2.527 2.527 0 0 1 2.521-2.52 2.527 2.527 0 0 1 2.521 2.52v6.313A2.528 2.528 0 0 1 8.834 24a2.528 2.528 0 0 1-2.521-2.522v-6.313z" fill="#E01E5A"/>
        <path d="M8.834 5.042a2.528 2.528 0 0 1-2.521-2.52A2.528 2.528 0 0 1 8.834 0a2.528 2.528 0 0 1 2.521 2.522v2.52H8.834zm0 1.271a2.528 2.528 0 0 1 2.521 2.521 2.528 2.528 0 0 1-2.521 2.521H2.522A2.528 2.528 0 0 1 0 8.834a2.528 2.528 0 0 1 2.522-2.521h6.312z" fill="#36C5F0"/>
        <path d="M18.956 8.834a2.528 2.528 0 0 1 2.522-2.521A2.528 2.528 0 0 1 24 8.834a2.528 2.528 0 0 1-2.522 2.521h-2.522V8.834zm-1.27 0a2.528 2.528 0 0 1-2.523 2.521 2.527 2.527 0 0 1-2.52-2.521V2.522A2.527 2.527 0 0 1 15.163 0a2.528 2.528 0 0 1 2.523 2.522v6.312z" fill="#2EB67D"/>
        <path d="M15.163 18.956a2.528 2.528 0 0 1 2.523 2.522A2.528 2.528 0 0 1 15.163 24a2.527 2.527 0 0 1-2.52-2.522v-2.522h2.52zm0-1.27a2.527 2.527 0 0 1-2.52-2.523 2.527 2.527 0 0 1 2.52-2.52h6.315A2.528 2.528 0 0 1 24 15.163a2.528 2.528 0 0 1-2.522 2.523h-6.315z" fill="#ECB22E"/>
      </g>

      {/* Webhook icon (left, top) */}
      <rect x="18" y="21" width="30" height="30" rx="8" fill="#111114" stroke="#C4693D" strokeWidth="1" opacity="0.9" />
      <g transform="translate(24, 27)">
        <circle cx="4" cy="13" r="2.5" stroke="#C4693D" strokeWidth="1.2" fill="none" />
        <circle cx="14" cy="13" r="2.5" stroke="#C4693D" strokeWidth="1.2" fill="none" />
        <circle cx="9" cy="3" r="2.5" stroke="#C4693D" strokeWidth="1.2" fill="none" />
        <path d="M9 5.5v3c0 1-1 2-2 2H5" stroke="#C4693D" strokeWidth="1.2" strokeLinecap="round" />
        <path d="M6.5 13h3c1 0 2-1 2-2V8.5" stroke="#C4693D" strokeWidth="1.2" strokeLinecap="round" />
      </g>

      {/* CIAB Server (center) */}
      <rect x="120" y="55" width="40" height="50" rx="10" fill="#18181B" stroke="#C4693D" strokeWidth="1.5" />
      <g transform="translate(126, 63)">
        <rect width="28" height="28" rx="7" fill="none" />
        {/* Simplified CIAB box logo */}
        <rect x="4" y="4" width="20" height="20" rx="4" fill="#C4693D" opacity="0.15" stroke="#C4693D" strokeWidth="0.8" />
        <circle cx="11" cy="13" r="1.5" fill="#C4693D" opacity="0.8" />
        <circle cx="17" cy="13" r="1.5" fill="#C4693D" opacity="0.8" />
        <path d="M10 18c0 0 2 2 4 0" stroke="#C4693D" strokeWidth="0.8" strokeLinecap="round" opacity="0.6" />
      </g>
      <text x="140" y="100" textAnchor="middle" fill="#C4693D" fontSize="6" fontFamily="monospace" opacity="0.6">CIAB</text>

      {/* Agent icons (right) */}
      <rect x="230" y="55" width="30" height="30" rx="8" fill="#111114" stroke="#D97757" strokeWidth="1" opacity="0.9" />
      <g transform="translate(237.5, 62.5) scale(0.625)">
        <path d="M17.3041 3.541h-3.6718l6.696 16.918H24Zm-10.6082 0L0 20.459h3.7442l1.3693-3.5527h7.0052l1.3693 3.5528h3.7442L10.5363 3.5409Zm-.3712 10.2232 2.2914-5.9456 2.2914 5.9456Z" fill="#D97757"/>
      </g>
      <rect x="230" y="90" width="30" height="30" rx="8" fill="#111114" stroke="#10A37F" strokeWidth="1" opacity="0.9" />
      <g transform="translate(237.5, 97.5) scale(0.625)">
        <path d="M22.282 9.821a5.985 5.985 0 0 0-.516-4.91 6.046 6.046 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.18a5.998 5.998 0 0 0-3.998 2.9 6.046 6.046 0 0 0 .743 7.097 5.98 5.98 0 0 0 .51 4.911 6.051 6.051 0 0 0 6.515 2.9A5.985 5.985 0 0 0 13.26 24a6.056 6.056 0 0 0 5.772-4.206 5.99 5.99 0 0 0 3.997-2.9 6.056 6.056 0 0 0-.747-7.073z" fill="#10A37F"/>
      </g>

      {/* Labels */}
      <text x="33" y="17" textAnchor="middle" fill="#52525B" fontSize="7" fontFamily="monospace">Platforms</text>
      <text x="140" y="48" textAnchor="middle" fill="#52525B" fontSize="7" fontFamily="monospace">Channel Router</text>
      <text x="245" y="51" textAnchor="middle" fill="#52525B" fontSize="7" fontFamily="monospace">Agents</text>

      {/* Animated pulse circles */}
      <circle cx="140" cy="80" r="3" fill="#C4693D" opacity="0.3">
        <animate attributeName="r" values="2;6;2" dur="2s" repeatCount="indefinite" />
        <animate attributeName="opacity" values="0.4;0;0.4" dur="2s" repeatCount="indefinite" />
      </circle>
    </svg>
  );
}

// =============================================================================
// Main Page
// =============================================================================

export default function ChannelList() {
  const { data: channelList, isLoading, isFetching } = useChannels();
  const deleteChannel = useDeleteChannel();
  const startChannel = useStartChannel();
  const stopChannel = useStopChannel();
  const qc = useQueryClient();
  const [showCreate, setShowCreate] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [spinning, setSpinning] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const filtered = searchQuery
    ? channelList?.filter(
        (ch) =>
          ch.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          ch.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
          ch.provider.includes(searchQuery.toLowerCase())
      )
    : channelList;

  const handleRefresh = () => {
    setSpinning(true);
    qc.invalidateQueries({ queryKey: ["channels"] });
    setTimeout(() => setSpinning(false), 600);
  };

  const copyWebhookUrl = (ch: Channel) => {
    const baseUrl = window.location.origin.replace(/:\d+$/, ':9090');
    const url = `${baseUrl}/api/v1/channels/webhook/${ch.id}/inbound`;
    navigator.clipboard.writeText(url);
    setCopiedId(ch.id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  // Stats
  const connectedCount = channelList?.filter((c) => c.state === "connected").length ?? 0;
  const totalCount = channelList?.length ?? 0;

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-ciab-copper/20 to-ciab-copper/5 border border-ciab-copper/20 flex items-center justify-center">
            <Zap className="w-5 h-5 text-ciab-copper" />
          </div>
          <div>
            <div className="flex items-center gap-2.5">
              <h1 className="text-xl font-semibold tracking-tight">Channels</h1>
              {connectedCount > 0 && (
                <span className="flex items-center gap-1.5 text-[10px] font-mono bg-emerald-500/10 text-emerald-500 px-2 py-0.5 rounded-full">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse-slow" />
                  {connectedCount} ACTIVE
                </span>
              )}
            </div>
            <p className="text-xs text-ciab-text-muted font-mono">
              Connect messaging platforms to your agents
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleRefresh}
            className="p-2 rounded-md text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
            title="Refresh"
          >
            <RefreshCw className={`w-4 h-4 ${spinning || isFetching ? "animate-spin" : ""}`} />
          </button>
          <button
            onClick={() => setShowCreate(true)}
            className="btn-primary flex items-center gap-2 text-sm"
          >
            <Plus className="w-4 h-4" />
            New Channel
          </button>
        </div>
      </div>

      {/* Quick Stats (when channels exist) */}
      {totalCount > 0 && (
        <div className="grid grid-cols-3 gap-3">
          <MiniStat label="Total" value={totalCount} color="text-ciab-text-primary" />
          <MiniStat label="Connected" value={connectedCount} color="text-emerald-500" />
          <MiniStat label="Inactive" value={totalCount - connectedCount} color="text-ciab-text-muted" />
        </div>
      )}

      {/* Search */}
      {totalCount > 3 && (
        <div className="flex items-center gap-2 bg-ciab-bg-secondary border border-ciab-border rounded-lg px-2.5 py-1.5 w-fit">
          <Search className="w-3 h-3 text-ciab-text-muted" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Filter by name, provider..."
            className="bg-transparent border-none outline-none text-[11px] text-ciab-text-primary placeholder:text-ciab-text-muted/40 w-56"
          />
          {searchQuery && (
            <button onClick={() => setSearchQuery("")} className="text-ciab-text-muted hover:text-ciab-text-secondary">
              <X className="w-3 h-3" />
            </button>
          )}
        </div>
      )}

      {/* Channel Grid */}
      {filtered && filtered.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
          {filtered.map((ch) => (
            <ChannelCard
              key={ch.id}
              channel={ch}
              onDelete={() => deleteChannel.mutate(ch.id)}
              onStart={() => startChannel.mutate(ch.id)}
              onStop={() => stopChannel.mutate(ch.id)}
              onCopyWebhook={() => copyWebhookUrl(ch)}
              isCopied={copiedId === ch.id}
            />
          ))}
        </div>
      ) : totalCount === 0 ? (
        /* Rich empty state */
        <div className="flex flex-col items-center justify-center py-10 text-center">
          <ChannelsEmptyIllustration />
          <h3 className="text-base font-semibold text-ciab-text-primary mt-4 mb-1.5">
            Route messages to your agents
          </h3>
          <p className="text-xs text-ciab-text-muted max-w-md mb-5 leading-relaxed">
            Connect WhatsApp, Slack, or webhook integrations to automatically route
            inbound messages to CIAB agents. Each channel can bind to a specific sandbox
            or auto-provision one from a workspace.
          </p>
          <div className="flex items-center gap-6 mb-6">
            <div className="flex flex-col items-center gap-1.5">
              <div className="w-10 h-10 rounded-lg bg-[#25D366]/10 border border-[#25D366]/20 flex items-center justify-center">
                <WhatsAppIcon size={20} />
              </div>
              <span className="text-[10px] text-ciab-text-muted font-mono">WhatsApp</span>
            </div>
            <div className="flex flex-col items-center gap-1.5">
              <div className="w-10 h-10 rounded-lg bg-[#611f69]/10 border border-[#611f69]/20 flex items-center justify-center">
                <SlackIcon size={20} />
              </div>
              <span className="text-[10px] text-ciab-text-muted font-mono">Slack</span>
            </div>
            <div className="flex flex-col items-center gap-1.5">
              <div className="w-10 h-10 rounded-lg bg-ciab-copper/10 border border-ciab-copper/20 flex items-center justify-center">
                <WebhookIcon size={20} />
              </div>
              <span className="text-[10px] text-ciab-text-muted font-mono">Webhooks</span>
            </div>
          </div>
          <button
            onClick={() => setShowCreate(true)}
            className="btn-primary flex items-center gap-2 text-sm"
          >
            <Plus className="w-4 h-4" />
            Create your first channel
          </button>
        </div>
      ) : (
        <p className="text-sm text-ciab-text-muted text-center py-6">
          No channels match &ldquo;{searchQuery}&rdquo;
        </p>
      )}

      {showCreate && (
        <CreateChannelDialog onClose={() => setShowCreate(false)} />
      )}
    </div>
  );
}

// =============================================================================
// Mini Stat
// =============================================================================

function MiniStat({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div className="rounded-lg border border-ciab-border bg-ciab-bg-card px-3 py-2.5">
      <div className="text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider mb-0.5">
        {label}
      </div>
      <div className={`text-lg font-semibold ${color}`}>{value}</div>
    </div>
  );
}

// =============================================================================
// Channel Card
// =============================================================================

function ChannelCard({
  channel,
  onDelete,
  onStart,
  onStop,
  onCopyWebhook,
  isCopied,
}: {
  channel: Channel;
  onDelete: () => void;
  onStart: () => void;
  onStop: () => void;
  onCopyWebhook: () => void;
  isCopied: boolean;
}) {
  const colors = PROVIDER_COLORS[channel.provider];
  const stateStyle = STATE_STYLES[channel.state] ?? STATE_STYLES.inactive;
  const isRunning = channel.state === "connected" || channel.state === "reconnecting";

  return (
    <Link
      to={`/channels/${channel.id}`}
      className="group relative rounded-xl border border-ciab-border bg-ciab-bg-card overflow-hidden transition-all hover:border-ciab-copper/30 hover:shadow-lg hover:shadow-ciab-copper/5"
    >
      {/* Provider color accent */}
      <div className={`h-1.5 bg-gradient-to-r ${colors.gradient} ${isRunning ? "opacity-100" : "opacity-50"} group-hover:opacity-100 transition-opacity`} />

      <div className="p-4">
        {/* Header */}
        <div className="flex items-start justify-between mb-3">
          <div className="flex items-center gap-2.5 min-w-0">
            <div className={`w-9 h-9 rounded-lg bg-gradient-to-br ${colors.gradient} border ${colors.border} flex items-center justify-center flex-shrink-0`}>
              <ChannelProviderIcon provider={channel.provider} size={18} />
            </div>
            <div className="min-w-0">
              <h3 className="font-medium text-sm text-ciab-text-primary truncate leading-tight">
                {channel.name}
              </h3>
              <div className="flex items-center gap-1.5 mt-0.5">
                <span className="text-[10px] text-ciab-text-muted font-mono">
                  {PROVIDER_LABELS[channel.provider]}
                </span>
              </div>
            </div>
          </div>
          <span
            className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[9px] font-mono ${stateStyle.bg} ${stateStyle.text} flex-shrink-0`}
          >
            <span className={`w-1.5 h-1.5 rounded-full bg-current ${stateStyle.dot ?? ""}`} />
            {stateStyle.label}
          </span>
        </div>

        {/* Description */}
        {channel.description && (
          <p className="text-xs text-ciab-text-secondary leading-relaxed mb-3 line-clamp-2">
            {channel.description}
          </p>
        )}

        {/* Binding info */}
        <div className="flex flex-wrap gap-1.5 mb-3">
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-ciab-bg-primary border border-ciab-border text-[10px] font-mono text-ciab-text-muted">
            {channel.binding.type === "static" ? (
              <><Link2 className="w-2.5 h-2.5" /> Static binding</>
            ) : (
              <><Layers className="w-2.5 h-2.5" /> Auto-provision</>
            )}
          </span>
          {channel.rules.rate_limit_per_minute && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-ciab-bg-primary border border-ciab-border text-[10px] font-mono text-ciab-text-muted">
              {channel.rules.rate_limit_per_minute}/min
            </span>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between pt-2 border-t border-ciab-border/50">
          <div className="flex items-center gap-2 text-[10px] font-mono text-ciab-text-muted">
            <span>{truncateId(channel.id)}</span>
            <span>{formatRelativeTime(channel.created_at)}</span>
          </div>
          <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            {channel.provider === "webhook" && (
              <button
                onClick={(e) => { e.preventDefault(); e.stopPropagation(); onCopyWebhook(); }}
                className="p-1 rounded text-ciab-text-muted hover:text-ciab-copper hover:bg-ciab-copper/10"
                title="Copy webhook URL"
              >
                {isCopied ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3" />}
              </button>
            )}
            {isRunning ? (
              <button
                onClick={(e) => { e.preventDefault(); e.stopPropagation(); onStop(); }}
                className="p-1 rounded text-ciab-text-muted hover:text-amber-500 hover:bg-amber-500/10"
                title="Stop"
              >
                <Square className="w-3 h-3" />
              </button>
            ) : (
              <button
                onClick={(e) => { e.preventDefault(); e.stopPropagation(); onStart(); }}
                className="p-1 rounded text-ciab-text-muted hover:text-emerald-500 hover:bg-emerald-500/10"
                title="Start"
              >
                <Play className="w-3 h-3" />
              </button>
            )}
            <button
              onClick={(e) => { e.preventDefault(); e.stopPropagation(); onDelete(); }}
              className="p-1 rounded text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10"
              title="Delete"
            >
              <Trash2 className="w-3 h-3" />
            </button>
          </div>
        </div>
      </div>
    </Link>
  );
}
