import { useState } from "react";
import {
  useCredentials,
  useCreateCredential,
  useDeleteCredential,
} from "@/lib/hooks/use-credentials";
import { KeyRound, Plus, Trash2, Shield, X } from "lucide-react";
import { formatRelativeTime } from "@/lib/utils/format";
import LoadingSpinner from "@/components/shared/LoadingSpinner";
import EmptyState from "@/components/shared/EmptyState";
import type { CreateCredentialRequest, CredentialType } from "@/lib/api/types";

export default function Credentials() {
  const { data: credentialList, isLoading } = useCredentials();
  const createCredential = useCreateCredential();
  const deleteCredential = useDeleteCredential();
  const [showCreate, setShowCreate] = useState(false);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="space-y-5 animate-fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">Credentials</h1>
          <p className="text-sm text-ciab-text-muted mt-0.5">
            Encrypted API keys and tokens
          </p>
        </div>
        <button
          onClick={() => setShowCreate(true)}
          className="btn-primary flex items-center gap-2"
        >
          <Plus className="w-4 h-4" />
          Add Credential
        </button>
      </div>

      {credentialList && credentialList.length > 0 ? (
        <div className="card overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="border-b border-ciab-border">
                <th className="text-left px-3 py-2 text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">Name</th>
                <th className="text-left px-3 py-2 text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">Type</th>
                <th className="text-left px-3 py-2 text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider">Created</th>
                <th className="w-12" />
              </tr>
            </thead>
            <tbody>
              {credentialList.map((cred) => (
                <tr
                  key={cred.id}
                  className="border-b border-ciab-border/50 last:border-0 hover:bg-ciab-bg-hover/30 transition-colors"
                >
                  <td className="px-3 py-2.5">
                    <div className="flex items-center gap-2">
                      <Shield className="w-3.5 h-3.5 text-ciab-copper" />
                      <span className="text-sm font-medium">{cred.name}</span>
                    </div>
                  </td>
                  <td className="px-3 py-2.5">
                    <span className="text-[10px] font-mono text-ciab-text-muted bg-ciab-bg-elevated px-1.5 py-0.5 rounded">
                      {cred.credential_type}
                    </span>
                  </td>
                  <td className="px-3 py-2.5 text-xs text-ciab-text-muted font-mono">
                    {formatRelativeTime(cred.created_at)}
                  </td>
                  <td className="px-3 py-2.5">
                    <button
                      onClick={() => deleteCredential.mutate(cred.id)}
                      className="p-1 rounded text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10 transition-colors"
                      title="Delete"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <EmptyState
          icon={KeyRound}
          title="No credentials"
          description="Store encrypted API keys and tokens for agent access."
          action={
            <button onClick={() => setShowCreate(true)} className="btn-primary">
              Add Credential
            </button>
          }
        />
      )}

      {showCreate && (
        <CreateCredentialDialog
          onClose={() => setShowCreate(false)}
          onCreate={(req) => {
            createCredential.mutate(req);
            setShowCreate(false);
          }}
        />
      )}
    </div>
  );
}

function CreateCredentialDialog({
  onClose,
  onCreate,
}: {
  onClose: () => void;
  onCreate: (req: CreateCredentialRequest) => void;
}) {
  const [name, setName] = useState("");
  const [type, setType] = useState<CredentialType>("api_key");
  const [value, setValue] = useState("");

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-end sm:items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-t-xl sm:rounded-lg w-full sm:max-w-md animate-scale-in"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between p-4 border-b border-ciab-border">
          <h2 className="text-sm font-semibold">Add Credential</h2>
          <button onClick={onClose} className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors p-0.5">
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="p-4 space-y-3">
          <div>
            <label className="label">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="anthropic-key"
              className="input w-full"
            />
          </div>

          <div>
            <label className="label">Type</label>
            <select
              value={type}
              onChange={(e) => setType(e.target.value as CredentialType)}
              className="input w-full"
            >
              <option value="api_key">API Key</option>
              <option value="env_vars">Environment Variables</option>
              <option value="git_token">Git Token</option>
              <option value="oauth_token">OAuth Token</option>
              <option value="ssh_key">SSH Key</option>
            </select>
          </div>

          <div>
            <label className="label">Secret Value</label>
            <input
              type="password"
              value={value}
              onChange={(e) => setValue(e.target.value)}
              placeholder="sk-ant-api03-..."
              className="input w-full font-mono text-xs"
            />
          </div>
        </div>

        <div className="flex justify-end gap-2 p-4 border-t border-ciab-border">
          <button onClick={onClose} className="btn-secondary text-sm px-3 py-1.5">
            Cancel
          </button>
          <button
            onClick={() =>
              onCreate({
                name,
                credential_type: type,
                value,
              })
            }
            disabled={!name.trim() || !value.trim()}
            className="btn-primary disabled:opacity-30 text-sm px-3 py-1.5"
          >
            Create
          </button>
        </div>
      </div>
    </div>
  );
}
