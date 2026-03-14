import React, { useState, useMemo } from "react";
import {
  X,
  ArrowRight,
  ArrowLeft,
  Search,
  ChevronDown,
  Link2,
  Layers,
  Clock,
  Bot,
} from "lucide-react";
import { useCreateChannel } from "@/lib/hooks/use-channels";
import { useSandboxes } from "@/lib/hooks/use-sandboxes";
import { useWorkspaces } from "@/lib/hooks/use-workspaces";
import { truncateId } from "@/lib/utils/format";
import { WhatsAppIcon, SlackIcon, WebhookIcon } from "@/pages/ChannelList";
import type {
  ChannelProvider,
  ChannelBinding,
  ChannelProviderConfig,
  CreateChannelRequest,
} from "@/lib/api/types";

interface Props {
  onClose: () => void;
}

const PROVIDERS: Array<{
  value: ChannelProvider;
  label: string;
  description: string;
  icon: ({ size }: { size?: number }) => React.ReactNode;
  accent: string;
  border: string;
}> = [
  {
    value: "webhook",
    label: "Webhook",
    description: "HTTP in/out — simplest integration for CI/CD and custom apps",
    icon: WebhookIcon,
    accent: "text-ciab-copper",
    border: "hover:border-ciab-copper/40",
  },
  {
    value: "slack",
    label: "Slack",
    description: "Connect a Slack bot to interact with agents in channels",
    icon: SlackIcon,
    accent: "text-[#611f69]",
    border: "hover:border-[#611f69]/40",
  },
  {
    value: "whatsapp",
    label: "WhatsApp",
    description: "Chat with agents from your phone via WhatsApp",
    icon: WhatsAppIcon,
    accent: "text-[#25D366]",
    border: "hover:border-[#25D366]/40",
  },
];

// =============================================================================
// Sandbox Picker (inline)
// =============================================================================

function SandboxPicker({
  value,
  onChange,
}: {
  value: string;
  onChange: (id: string) => void;
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
      <label className="label flex items-center gap-1.5">
        <Bot className="w-3 h-3" />
        Sandbox
      </label>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="input w-full text-left flex items-center justify-between"
      >
        {selected ? (
          <div className="flex items-center gap-2 min-w-0">
            <span className="text-sm truncate">
              {selected.name ?? truncateId(selected.id)}
            </span>
            <span className={`px-1.5 py-0.5 rounded text-[9px] font-mono ${
              selected.state === "running" ? "bg-emerald-500/10 text-emerald-500" : "bg-ciab-bg-elevated text-ciab-text-muted"
            }`}>
              {selected.state}
            </span>
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
          <div className="absolute top-full left-0 right-0 mt-1 bg-ciab-bg-card border border-ciab-border rounded-lg shadow-lg z-50 animate-slide-down max-h-52 flex flex-col">
            <div className="p-2 border-b border-ciab-border/50">
              <div className="relative">
                <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-ciab-text-muted" />
                <input
                  type="text"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  placeholder="Search sandboxes..."
                  className="input w-full pl-7 py-1.5 text-xs"
                  autoFocus
                />
              </div>
            </div>
            <div className="overflow-y-auto">
              {isLoading ? (
                <div className="p-3 space-y-2">
                  {[...Array(3)].map((_, i) => (
                    <div key={i} className="bg-ciab-bg-elevated rounded animate-pulse h-8" />
                  ))}
                </div>
              ) : filtered.length === 0 ? (
                <div className="p-4 text-center text-xs text-ciab-text-muted">
                  {search ? "No matches" : "No sandboxes available"}
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
                    <div className="flex-1 min-w-0">
                      <div className="text-sm truncate">{sb.name ?? truncateId(sb.id)}</div>
                      <div className="text-[10px] font-mono text-ciab-text-muted truncate">
                        {truncateId(sb.id)} · {sb.agent_provider}
                      </div>
                    </div>
                    <span className={`px-1.5 py-0.5 rounded text-[9px] font-mono flex-shrink-0 ${
                      sb.state === "running" ? "bg-emerald-500/10 text-emerald-500" : "bg-ciab-bg-elevated text-ciab-text-muted"
                    }`}>
                      {sb.state}
                    </span>
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
// Workspace Picker (inline)
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
      <label className="label flex items-center gap-1.5">
        <Layers className="w-3 h-3" />
        Workspace
      </label>
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
          <div className="absolute top-full left-0 right-0 mt-1 bg-ciab-bg-card border border-ciab-border rounded-lg shadow-lg z-50 animate-slide-down max-h-52 flex flex-col">
            <div className="p-2 border-b border-ciab-border/50">
              <div className="relative">
                <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-ciab-text-muted" />
                <input
                  type="text"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  placeholder="Search workspaces..."
                  className="input w-full pl-7 py-1.5 text-xs"
                  autoFocus
                />
              </div>
            </div>
            <div className="overflow-y-auto">
              {isLoading ? (
                <div className="p-3 space-y-2">
                  {[...Array(3)].map((_, i) => (
                    <div key={i} className="bg-ciab-bg-elevated rounded animate-pulse h-8" />
                  ))}
                </div>
              ) : filtered.length === 0 ? (
                <div className="p-4 text-center text-xs text-ciab-text-muted">
                  {search ? "No matches" : "No workspaces available"}
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
// CreateChannelDialog
// =============================================================================

export default function CreateChannelDialog({ onClose }: Props) {
  const createChannel = useCreateChannel();
  const [step, setStep] = useState<"provider" | "configure">("provider");
  const [provider, setProvider] = useState<ChannelProvider>("webhook");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [bindingType, setBindingType] = useState<"static" | "auto_provision">("static");
  const [sandboxId, setSandboxId] = useState("");
  const [workspaceId, setWorkspaceId] = useState("");
  const [ttlSecs, setTtlSecs] = useState(3600);

  // Webhook config
  const [outboundUrl, setOutboundUrl] = useState("");
  const [inboundSecret, setInboundSecret] = useState("");

  // Slack config
  const [botToken, setBotToken] = useState("");
  const [appToken, setAppToken] = useState("");

  const providerInfo = PROVIDERS.find((p) => p.value === provider);

  const handleSelectProvider = (p: ChannelProvider) => {
    setProvider(p);
    setName(`my-${p}-channel`);
    setStep("configure");
  };

  const canSubmit = () => {
    if (!name.trim()) return false;
    if (bindingType === "static" && !sandboxId) return false;
    if (bindingType === "auto_provision" && !workspaceId) return false;
    if (provider === "slack" && !botToken.trim()) return false;
    return true;
  };

  const handleSubmit = () => {
    const binding: ChannelBinding =
      bindingType === "static"
        ? { type: "static", sandbox_id: sandboxId }
        : {
            type: "auto_provision",
            workspace_id: workspaceId,
            ttl_secs: ttlSecs,
          };

    let provider_config: ChannelProviderConfig;
    switch (provider) {
      case "webhook":
        provider_config = {
          provider: "webhook",
          outbound_url: outboundUrl || undefined,
          inbound_secret: inboundSecret || undefined,
          outbound_headers: {},
        };
        break;
      case "slack":
        provider_config = {
          provider: "slack",
          bot_token: botToken,
          app_token: appToken || undefined,
        };
        break;
      case "whatsapp":
        provider_config = { provider: "whatsapp" };
        break;
    }

    const request: CreateChannelRequest = {
      name: name.trim(),
      description: description.trim() || undefined,
      provider,
      binding,
      provider_config,
    };

    createChannel.mutate(request, { onSuccess: onClose });
  };

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-end sm:items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-t-xl sm:rounded-xl w-full sm:max-w-lg max-h-[90vh] flex flex-col animate-scale-in overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="h-1 bg-gradient-to-r from-ciab-copper to-ciab-copper-light" />

        {/* Header */}
        <div className="flex items-center justify-between px-5 pt-4 pb-2">
          <div className="flex items-center gap-2.5">
            {step === "configure" && providerInfo && (
              <providerInfo.icon size={18} />
            )}
            <h2 className="text-sm font-semibold">
              {step === "provider"
                ? "New Channel"
                : `New ${providerInfo?.label} Channel`}
            </h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded text-ciab-text-muted hover:text-ciab-text-primary transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="px-5 pb-5 overflow-y-auto space-y-4">
          {step === "provider" ? (
            <div className="space-y-2">
              <p className="text-xs text-ciab-text-muted">Choose a messaging platform</p>
              {PROVIDERS.map((p) => (
                <button
                  key={p.value}
                  onClick={() => handleSelectProvider(p.value)}
                  className={`w-full flex items-center gap-3 p-3.5 rounded-lg border border-ciab-border ${p.border} hover:bg-ciab-bg-hover transition-all text-left group`}
                >
                  <div className="w-10 h-10 rounded-lg bg-ciab-bg-elevated flex items-center justify-center flex-shrink-0 group-hover:scale-105 transition-transform">
                    <p.icon size={22} />
                  </div>
                  <div className="flex-1">
                    <div className="text-sm font-medium">{p.label}</div>
                    <div className="text-[11px] text-ciab-text-muted mt-0.5">{p.description}</div>
                  </div>
                  <ArrowRight className="w-4 h-4 text-ciab-text-muted opacity-0 group-hover:opacity-100 transition-opacity" />
                </button>
              ))}
            </div>
          ) : (
            <div className="space-y-4">
              <button
                onClick={() => setStep("provider")}
                className="flex items-center gap-1 text-xs text-ciab-text-muted hover:text-ciab-text-primary transition-colors"
              >
                <ArrowLeft className="w-3 h-3" />
                Back
              </button>

              {/* Name & Description */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label className="label">Channel Name</label>
                  <input
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    className="input w-full"
                    placeholder="my-webhook-channel"
                    autoFocus
                  />
                </div>
                <div>
                  <label className="label">Description (optional)</label>
                  <input
                    type="text"
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    className="input w-full"
                    placeholder="What's this channel for?"
                  />
                </div>
              </div>

              {/* Binding */}
              <div className="rounded-lg border border-ciab-border bg-ciab-bg-primary p-4 space-y-3">
                <div className="flex items-center gap-1.5 text-xs font-medium">
                  <Link2 className="w-3.5 h-3.5 text-ciab-text-muted" />
                  Agent Binding
                </div>

                <div className="flex gap-2">
                  <button
                    onClick={() => setBindingType("static")}
                    className={`flex-1 px-3 py-2 rounded-lg text-xs font-medium border transition-all ${
                      bindingType === "static"
                        ? "border-ciab-copper bg-ciab-copper/10 text-ciab-copper shadow-sm"
                        : "border-ciab-border text-ciab-text-muted hover:bg-ciab-bg-hover"
                    }`}
                  >
                    <div className="font-medium">Static Sandbox</div>
                    <div className="text-[10px] mt-0.5 opacity-70">Bind to an existing sandbox</div>
                  </button>
                  <button
                    onClick={() => setBindingType("auto_provision")}
                    className={`flex-1 px-3 py-2 rounded-lg text-xs font-medium border transition-all ${
                      bindingType === "auto_provision"
                        ? "border-ciab-copper bg-ciab-copper/10 text-ciab-copper shadow-sm"
                        : "border-ciab-border text-ciab-text-muted hover:bg-ciab-bg-hover"
                    }`}
                  >
                    <div className="font-medium">Auto-Provision</div>
                    <div className="text-[10px] mt-0.5 opacity-70">Create sandboxes on demand</div>
                  </button>
                </div>

                {bindingType === "static" ? (
                  <SandboxPicker value={sandboxId} onChange={setSandboxId} />
                ) : (
                  <>
                    <WorkspacePicker value={workspaceId} onChange={setWorkspaceId} />
                    <div>
                      <label className="label flex items-center gap-1.5">
                        <Clock className="w-3 h-3" />
                        Sandbox TTL
                      </label>
                      <div className="flex items-center gap-2">
                        <input
                          type="range"
                          min={300}
                          max={86400}
                          step={300}
                          value={ttlSecs}
                          onChange={(e) => setTtlSecs(Number(e.target.value))}
                          className="flex-1 accent-ciab-copper"
                        />
                        <span className="text-xs font-mono text-ciab-text-secondary w-16 text-right">
                          {ttlSecs >= 3600
                            ? `${(ttlSecs / 3600).toFixed(1)}h`
                            : `${Math.round(ttlSecs / 60)}m`}
                        </span>
                      </div>
                    </div>
                  </>
                )}
              </div>

              {/* Provider-specific config */}
              {provider === "webhook" && (
                <div className="rounded-lg border border-ciab-border bg-ciab-bg-primary p-4 space-y-3">
                  <div className="text-xs font-medium flex items-center gap-1.5">
                    <WebhookIcon size={14} />
                    Webhook Configuration
                  </div>
                  <div>
                    <label className="label">Outbound URL (optional)</label>
                    <input
                      type="text"
                      value={outboundUrl}
                      onChange={(e) => setOutboundUrl(e.target.value)}
                      className="input w-full"
                      placeholder="https://your-server.com/webhook"
                    />
                    <p className="text-[10px] text-ciab-text-muted mt-1">Agent responses will be POSTed here</p>
                  </div>
                  <div>
                    <label className="label">HMAC Secret (optional)</label>
                    <input
                      type="password"
                      value={inboundSecret}
                      onChange={(e) => setInboundSecret(e.target.value)}
                      className="input w-full font-mono"
                      placeholder="secret-key"
                    />
                    <p className="text-[10px] text-ciab-text-muted mt-1">Used to verify inbound webhook signatures</p>
                  </div>
                </div>
              )}

              {provider === "slack" && (
                <div className="rounded-lg border border-ciab-border bg-ciab-bg-primary p-4 space-y-3">
                  <div className="text-xs font-medium flex items-center gap-1.5">
                    <SlackIcon size={14} />
                    Slack Configuration
                  </div>
                  <div>
                    <label className="label">Bot Token <span className="text-state-failed">*</span></label>
                    <input
                      type="password"
                      value={botToken}
                      onChange={(e) => setBotToken(e.target.value)}
                      className="input w-full font-mono"
                      placeholder="xoxb-..."
                    />
                    <p className="text-[10px] text-ciab-text-muted mt-1">From your Slack app's OAuth & Permissions page</p>
                  </div>
                  <div>
                    <label className="label">App Token (optional)</label>
                    <input
                      type="password"
                      value={appToken}
                      onChange={(e) => setAppToken(e.target.value)}
                      className="input w-full font-mono"
                      placeholder="xapp-..."
                    />
                    <p className="text-[10px] text-ciab-text-muted mt-1">Required for Socket Mode (recommended)</p>
                  </div>
                </div>
              )}

              {provider === "whatsapp" && (
                <div className="rounded-lg border border-[#25D366]/20 bg-[#25D366]/5 p-4 flex items-start gap-3">
                  <WhatsAppIcon size={20} />
                  <div>
                    <p className="text-xs text-ciab-text-secondary font-medium">QR Code Pairing</p>
                    <p className="text-[11px] text-ciab-text-muted mt-0.5">
                      After creating the channel, click Start to generate a QR code. Scan it with WhatsApp to link your device.
                    </p>
                  </div>
                </div>
              )}

              {/* Submit */}
              <div className="flex items-center justify-end gap-2 pt-2 border-t border-ciab-border">
                <button onClick={onClose} className="btn-ghost text-xs px-3 py-1.5">
                  Cancel
                </button>
                <button
                  onClick={handleSubmit}
                  disabled={!canSubmit() || createChannel.isPending}
                  className="btn-primary flex items-center gap-2 text-sm px-4 py-2 disabled:opacity-30"
                >
                  {createChannel.isPending ? (
                    "Creating..."
                  ) : (
                    <>
                      Create Channel
                      <ArrowRight className="w-3.5 h-3.5" />
                    </>
                  )}
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
