import { useState } from "react";
import { useParams, useNavigate } from "react-router";
import {
  ArrowLeft,
  Play,
  Square,
  RotateCw,
  Trash2,
  MessageSquare,
  Settings2,
  QrCode,
  Save,
  Undo2,
  Copy,
  Check,
  Shield,
  Link2,
  Clock,
  ChevronDown,
  ChevronUp,
  ExternalLink,
  ArrowDownLeft,
  ArrowUpRight,
} from "lucide-react";
import {
  useChannel,
  useUpdateChannel,
  useDeleteChannel,
  useStartChannel,
  useStopChannel,
  useRestartChannel,
  useChannelMessages,
  useChannelQr,
} from "@/lib/hooks/use-channels";
import { formatRelativeTime, truncateId } from "@/lib/utils/format";
import LoadingSpinner from "@/components/shared/LoadingSpinner";
import { ChannelProviderIcon } from "@/pages/ChannelList";
import type { ChannelState } from "@/lib/api/types";

const STATE_STYLES: Record<ChannelState, { bg: string; text: string; label: string; dot?: string }> = {
  inactive: { bg: "bg-ciab-bg-elevated", text: "text-ciab-text-muted", label: "Inactive" },
  pairing: { bg: "bg-amber-500/10", text: "text-amber-500", label: "Pairing", dot: "animate-pulse" },
  connected: { bg: "bg-emerald-500/10", text: "text-emerald-500", label: "Connected", dot: "animate-pulse-slow" },
  reconnecting: { bg: "bg-amber-500/10", text: "text-amber-500", label: "Reconnecting", dot: "animate-pulse" },
  failed: { bg: "bg-state-failed/10", text: "text-state-failed", label: "Failed" },
  stopped: { bg: "bg-ciab-bg-elevated", text: "text-ciab-text-muted", label: "Stopped" },
};

const PROVIDER_LABELS: Record<string, string> = {
  whatsapp: "WhatsApp",
  slack: "Slack",
  webhook: "Webhook",
};

export default function ChannelDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: channel, isLoading } = useChannel(id);
  const updateChannel = useUpdateChannel();
  const deleteChannel = useDeleteChannel();
  const startChannel = useStartChannel();
  const stopChannel = useStopChannel();
  const restartChannel = useRestartChannel();
  const { data: messages } = useChannelMessages(id, { limit: 50 });
  const isPairing = channel?.state === "pairing";
  const { data: qrData } = useChannelQr(id, isPairing);

  const [editName, setEditName] = useState<string | null>(null);
  const [editDescription, setEditDescription] = useState<string | null>(null);
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [rulesExpanded, setRulesExpanded] = useState(false);
  const hasEdits = editName !== null || editDescription !== null;

  const copyToClipboard = (text: string, field: string) => {
    navigator.clipboard.writeText(text);
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  if (!channel) {
    return (
      <div className="text-center py-12 text-ciab-text-muted">
        Channel not found
      </div>
    );
  }

  const stateStyle = STATE_STYLES[channel.state] ?? STATE_STYLES.inactive;
  const isRunning = channel.state === "connected" || channel.state === "reconnecting";
  const currentName = editName ?? channel.name;
  const currentDescription = editDescription ?? channel.description ?? "";
  const webhookUrl = channel.provider === "webhook" ? `/api/channels/webhook/${channel.id}/inbound` : null;

  const handleSave = () => {
    updateChannel.mutate({
      id: channel.id,
      ...(editName !== null ? { name: editName } : {}),
      ...(editDescription !== null ? { description: editDescription } : {}),
    });
    setEditName(null);
    setEditDescription(null);
  };

  const handleDelete = () => {
    deleteChannel.mutate(channel.id, {
      onSuccess: () => navigate("/channels"),
    });
  };

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <button
            onClick={() => navigate("/channels")}
            className="p-1.5 rounded-md text-ciab-text-muted hover:text-ciab-text-primary hover:bg-ciab-bg-hover transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div className="w-10 h-10 rounded-xl bg-ciab-bg-elevated flex items-center justify-center border border-ciab-border">
            <ChannelProviderIcon provider={channel.provider} size={22} />
          </div>
          <div>
            <h1 className="text-xl font-semibold tracking-tight">{channel.name}</h1>
            <div className="flex items-center gap-2 mt-0.5">
              <span
                className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-medium ${stateStyle.bg} ${stateStyle.text}`}
              >
                <span className={`w-1.5 h-1.5 rounded-full bg-current ${stateStyle.dot ?? ""}`} />
                {stateStyle.label}
              </span>
              <span className="text-xs text-ciab-text-muted">
                {PROVIDER_LABELS[channel.provider]}
              </span>
              {channel.description && (
                <>
                  <span className="text-ciab-text-muted">·</span>
                  <span className="text-xs text-ciab-text-muted truncate max-w-[200px]">
                    {channel.description}
                  </span>
                </>
              )}
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {hasEdits && (
            <>
              <button
                onClick={() => { setEditName(null); setEditDescription(null); }}
                className="btn-ghost flex items-center gap-1.5 text-xs"
              >
                <Undo2 className="w-3.5 h-3.5" />
                Discard
              </button>
              <button
                onClick={handleSave}
                disabled={updateChannel.isPending}
                className="btn-primary flex items-center gap-1.5 text-xs"
              >
                <Save className="w-3.5 h-3.5" />
                Save
              </button>
            </>
          )}
          {!hasEdits && (
            <>
              {isRunning ? (
                <>
                  <button
                    onClick={() => restartChannel.mutate(channel.id)}
                    className="btn-ghost flex items-center gap-1.5 text-xs"
                  >
                    <RotateCw className="w-3.5 h-3.5" />
                    Restart
                  </button>
                  <button
                    onClick={() => stopChannel.mutate(channel.id)}
                    className="btn-ghost flex items-center gap-1.5 text-xs text-amber-500"
                  >
                    <Square className="w-3.5 h-3.5" />
                    Stop
                  </button>
                </>
              ) : (
                <button
                  onClick={() => startChannel.mutate(channel.id)}
                  className="btn-primary flex items-center gap-1.5 text-xs"
                >
                  <Play className="w-3.5 h-3.5" />
                  Start
                </button>
              )}
              <button
                onClick={handleDelete}
                className="btn-ghost text-state-failed flex items-center gap-1.5 text-xs"
              >
                <Trash2 className="w-3.5 h-3.5" />
                Delete
              </button>
            </>
          )}
        </div>
      </div>

      {/* Error message */}
      {channel.error_message && (
        <div className="rounded-lg border border-state-failed/20 bg-state-failed/5 px-4 py-3 text-sm text-state-failed flex items-start gap-2">
          <span className="w-1.5 h-1.5 rounded-full bg-state-failed mt-1.5 flex-shrink-0" />
          {channel.error_message}
        </div>
      )}

      {/* QR Code for WhatsApp */}
      {channel.provider === "whatsapp" && isPairing && qrData?.qr_code && (
        <div className="rounded-xl border border-[#25D366]/20 bg-[#25D366]/5 p-6 flex flex-col items-center gap-4">
          <div className="flex items-center gap-2">
            <QrCode className="w-5 h-5 text-[#25D366]" />
            <span className="text-sm font-medium text-[#25D366]">WhatsApp Pairing</span>
          </div>
          <p className="text-xs text-ciab-text-muted">Open WhatsApp on your phone, go to Settings → Linked Devices → Link a Device</p>
          <div className="bg-white p-4 rounded-xl shadow-sm">
            <pre className="text-xs font-mono whitespace-pre text-black">{qrData.qr_code}</pre>
          </div>
          <div className="flex items-center gap-1.5 text-[10px] text-ciab-text-muted">
            <span className="w-1.5 h-1.5 rounded-full bg-amber-500 animate-pulse" />
            Waiting for scan...
          </div>
        </div>
      )}

      {/* Webhook URL banner */}
      {webhookUrl && (
        <div className="rounded-lg border border-ciab-copper/20 bg-ciab-copper/5 px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-2 min-w-0">
            <ExternalLink className="w-3.5 h-3.5 text-ciab-copper flex-shrink-0" />
            <span className="text-xs text-ciab-text-muted">Inbound URL</span>
            <code className="text-xs font-mono text-ciab-text-secondary truncate">{webhookUrl}</code>
          </div>
          <button
            onClick={() => copyToClipboard(webhookUrl, "webhook-url")}
            className="p-1 rounded text-ciab-text-muted hover:text-ciab-text-primary transition-colors flex-shrink-0"
          >
            {copiedField === "webhook-url" ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
          </button>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Settings Card */}
        <div className="rounded-xl border border-ciab-border bg-ciab-bg-card p-5 space-y-4">
          <div className="flex items-center gap-2">
            <Settings2 className="w-4 h-4 text-ciab-text-muted" />
            <h2 className="text-sm font-medium">Settings</h2>
          </div>

          <div className="space-y-3">
            <div>
              <label className="label">Name</label>
              <input
                type="text"
                value={currentName}
                onChange={(e) => setEditName(e.target.value)}
                className="input w-full"
              />
            </div>
            <div>
              <label className="label">Description</label>
              <input
                type="text"
                value={currentDescription}
                onChange={(e) => setEditDescription(e.target.value)}
                className="input w-full"
                placeholder="Optional description"
              />
            </div>

            {/* Binding */}
            <div>
              <label className="label flex items-center gap-1.5">
                <Link2 className="w-3 h-3" />
                Binding
              </label>
              <div className="rounded-lg bg-ciab-bg-primary border border-ciab-border p-3 space-y-1.5">
                <div className="flex items-center gap-2">
                  <span className={`px-1.5 py-0.5 rounded text-[9px] font-mono font-medium ${
                    channel.binding.type === "static"
                      ? "bg-ciab-copper/10 text-ciab-copper"
                      : "bg-emerald-500/10 text-emerald-500"
                  }`}>
                    {channel.binding.type === "static" ? "STATIC" : "AUTO-PROVISION"}
                  </span>
                </div>
                {channel.binding.type === "static" ? (
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-ciab-text-muted">Sandbox</span>
                    <code className="text-xs font-mono text-ciab-text-secondary">{truncateId(channel.binding.sandbox_id ?? "")}</code>
                    <button
                      onClick={() => copyToClipboard(channel.binding.sandbox_id ?? "", "sandbox-id")}
                      className="p-0.5 rounded text-ciab-text-muted hover:text-ciab-text-primary"
                    >
                      {copiedField === "sandbox-id" ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3" />}
                    </button>
                  </div>
                ) : (
                  <>
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-ciab-text-muted">Workspace</span>
                      <code className="text-xs font-mono text-ciab-text-secondary">{truncateId(channel.binding.workspace_id ?? "")}</code>
                    </div>
                    <div className="flex items-center gap-1.5 text-xs text-ciab-text-muted">
                      <Clock className="w-3 h-3" />
                      TTL: {channel.binding.ttl_secs}s
                    </div>
                  </>
                )}
              </div>
            </div>

            {/* ID */}
            <div className="flex items-center gap-2 text-[10px] text-ciab-text-muted font-mono pt-2 border-t border-ciab-border">
              <span>ID: {truncateId(channel.id)}</span>
              <button
                onClick={() => copyToClipboard(channel.id, "channel-id")}
                className="p-0.5 rounded hover:text-ciab-text-primary"
              >
                {copiedField === "channel-id" ? <Check className="w-2.5 h-2.5 text-emerald-500" /> : <Copy className="w-2.5 h-2.5" />}
              </button>
              <span className="ml-auto">
                Created {formatRelativeTime(channel.created_at)} · Updated{" "}
                {formatRelativeTime(channel.updated_at)}
              </span>
            </div>
          </div>
        </div>

        {/* Rules Card */}
        <div className="rounded-xl border border-ciab-border bg-ciab-bg-card p-5 space-y-4">
          <button
            onClick={() => setRulesExpanded(!rulesExpanded)}
            className="w-full flex items-center justify-between"
          >
            <div className="flex items-center gap-2">
              <Shield className="w-4 h-4 text-ciab-text-muted" />
              <h2 className="text-sm font-medium">Rules & Policies</h2>
            </div>
            {rulesExpanded ? (
              <ChevronUp className="w-3.5 h-3.5 text-ciab-text-muted" />
            ) : (
              <ChevronDown className="w-3.5 h-3.5 text-ciab-text-muted" />
            )}
          </button>

          <div className="grid grid-cols-2 gap-3">
            <div className="rounded-lg bg-ciab-bg-primary border border-ciab-border px-3 py-2">
              <div className="text-[10px] text-ciab-text-muted uppercase tracking-wider">DM Policy</div>
              <div className="text-sm font-medium mt-0.5 capitalize">{channel.rules.dm_policy ?? "respond"}</div>
            </div>
            <div className="rounded-lg bg-ciab-bg-primary border border-ciab-border px-3 py-2">
              <div className="text-[10px] text-ciab-text-muted uppercase tracking-wider">Group Policy</div>
              <div className="text-sm font-medium mt-0.5 capitalize">{(channel.rules.group_policy ?? "mention_only").replace("_", " ")}</div>
            </div>
          </div>

          {rulesExpanded && (
            <div className="space-y-2 animate-fade-in">
              {channel.rules.rate_limit_per_minute && (
                <div className="flex items-center justify-between text-xs px-3 py-2 rounded-lg bg-ciab-bg-primary border border-ciab-border">
                  <span className="text-ciab-text-muted">Rate limit</span>
                  <span className="font-mono">{channel.rules.rate_limit_per_minute}/min</span>
                </div>
              )}
              {channel.rules.max_message_length && (
                <div className="flex items-center justify-between text-xs px-3 py-2 rounded-lg bg-ciab-bg-primary border border-ciab-border">
                  <span className="text-ciab-text-muted">Max message length</span>
                  <span className="font-mono">{channel.rules.max_message_length} chars</span>
                </div>
              )}
              {channel.rules.reset_trigger && (
                <div className="flex items-center justify-between text-xs px-3 py-2 rounded-lg bg-ciab-bg-primary border border-ciab-border">
                  <span className="text-ciab-text-muted">Reset trigger</span>
                  <code className="font-mono text-ciab-copper">{channel.rules.reset_trigger}</code>
                </div>
              )}
              {(channel.rules.allowed_senders?.length ?? 0) > 0 && (
                <div className="text-xs px-3 py-2 rounded-lg bg-ciab-bg-primary border border-ciab-border">
                  <div className="text-ciab-text-muted mb-1">Allowed senders</div>
                  <div className="flex flex-wrap gap-1">
                    {channel.rules.allowed_senders!.map((s) => (
                      <span key={s} className="px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-500 text-[10px] font-mono">{s}</span>
                    ))}
                  </div>
                </div>
              )}
              {(channel.rules.blocked_senders?.length ?? 0) > 0 && (
                <div className="text-xs px-3 py-2 rounded-lg bg-ciab-bg-primary border border-ciab-border">
                  <div className="text-ciab-text-muted mb-1">Blocked senders</div>
                  <div className="flex flex-wrap gap-1">
                    {channel.rules.blocked_senders!.map((s) => (
                      <span key={s} className="px-1.5 py-0.5 rounded bg-state-failed/10 text-state-failed text-[10px] font-mono">{s}</span>
                    ))}
                  </div>
                </div>
              )}
              {channel.rules.persist_conversation !== undefined && (
                <div className="flex items-center justify-between text-xs px-3 py-2 rounded-lg bg-ciab-bg-primary border border-ciab-border">
                  <span className="text-ciab-text-muted">Persist conversation</span>
                  <span className={channel.rules.persist_conversation ? "text-emerald-500" : "text-ciab-text-muted"}>
                    {channel.rules.persist_conversation ? "Yes" : "No"}
                  </span>
                </div>
              )}
              {!channel.rules.rate_limit_per_minute && !channel.rules.max_message_length && !channel.rules.reset_trigger &&
               !(channel.rules.allowed_senders?.length) && !(channel.rules.blocked_senders?.length) && (
                <p className="text-xs text-ciab-text-muted text-center py-2">No additional rules configured</p>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Messages */}
      <div className="rounded-xl border border-ciab-border bg-ciab-bg-card p-5 space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <MessageSquare className="w-4 h-4 text-ciab-text-muted" />
            <h2 className="text-sm font-medium">Messages</h2>
          </div>
          {messages && (
            <span className="text-[10px] font-mono text-ciab-text-muted px-2 py-0.5 rounded-full bg-ciab-bg-elevated">
              {messages.length} messages
            </span>
          )}
        </div>

        <div className="space-y-2 max-h-[500px] overflow-y-auto pr-1">
          {messages && messages.length > 0 ? (
            messages.map((msg) => (
              <div
                key={msg.id}
                className={`rounded-lg px-4 py-3 text-xs ${
                  msg.direction === "inbound"
                    ? "bg-ciab-bg-primary border border-ciab-border"
                    : "bg-ciab-copper/5 border border-ciab-copper/10 ml-8"
                }`}
              >
                <div className="flex items-center justify-between mb-1.5">
                  <div className="flex items-center gap-1.5">
                    {msg.direction === "inbound" ? (
                      <ArrowDownLeft className="w-3 h-3 text-ciab-text-muted" />
                    ) : (
                      <ArrowUpRight className="w-3 h-3 text-ciab-copper" />
                    )}
                    <span className="font-medium text-ciab-text-secondary">
                      {msg.sender_name ?? msg.sender_id}
                    </span>
                  </div>
                  <span className="text-[9px] text-ciab-text-muted font-mono">
                    {formatRelativeTime(msg.timestamp)}
                  </span>
                </div>
                <p className="text-ciab-text-secondary whitespace-pre-wrap break-words leading-relaxed">
                  {msg.content}
                </p>
              </div>
            ))
          ) : (
            <div className="text-center py-10">
              <MessageSquare className="w-8 h-8 text-ciab-text-muted/30 mx-auto mb-2" />
              <p className="text-xs text-ciab-text-muted">No messages yet</p>
              <p className="text-[10px] text-ciab-text-muted/60 mt-1">
                {channel.state === "connected"
                  ? "Messages will appear here when the channel receives them"
                  : "Start the channel to begin receiving messages"}
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
