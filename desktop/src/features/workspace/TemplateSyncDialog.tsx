import { useState } from "react";
import {
  X,
  GitBranch,
  RefreshCw,
  Trash2,
  Plus,
  Loader2,
  Cloud,
  Clock,
  FolderGit2,
} from "lucide-react";
import {
  useTemplateSources,
  useAddTemplateSource,
  useDeleteTemplateSource,
  useSyncTemplateSource,
} from "@/lib/hooks/use-templates";
import { formatRelativeTime } from "@/lib/utils/format";

interface Props {
  onClose: () => void;
}

export default function TemplateSyncDialog({ onClose }: Props) {
  const { data: sources, isLoading } = useTemplateSources();
  const addSource = useAddTemplateSource();
  const deleteSource = useDeleteTemplateSource();
  const syncSource = useSyncTemplateSource();
  const [showAdd, setShowAdd] = useState(false);
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [branch, setBranch] = useState("main");
  const [templatesPath, setTemplatesPath] = useState(".ciab/templates");
  const [syncingId, setSyncingId] = useState<string | null>(null);

  const handleAdd = () => {
    if (!name.trim() || !url.trim()) return;
    addSource.mutate(
      {
        name: name.trim(),
        url: url.trim(),
        branch: branch.trim() || undefined,
        templates_path: templatesPath.trim() || undefined,
      },
      {
        onSuccess: (data) => {
          setName("");
          setUrl("");
          setBranch("main");
          setTemplatesPath(".ciab/templates");
          setShowAdd(false);
          // Auto-sync after adding
          const source = data as { id: string };
          if (source?.id) {
            setSyncingId(source.id);
            syncSource.mutate(source.id, {
              onSettled: () => setSyncingId(null),
            });
          }
        },
      }
    );
  };

  const handleSync = (id: string) => {
    setSyncingId(id);
    syncSource.mutate(id, {
      onSettled: () => setSyncingId(null),
    });
  };

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-end sm:items-center justify-center z-50 animate-fade-in"
      onClick={onClose}
    >
      <div
        className="bg-ciab-bg-card border border-ciab-border rounded-t-xl sm:rounded-xl w-full sm:max-w-lg max-h-[90vh] sm:max-h-[80vh] flex flex-col animate-scale-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-ciab-border">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-ciab-steel-blue/10 flex items-center justify-center">
              <Cloud className="w-4 h-4 text-ciab-steel-blue" />
            </div>
            <div>
              <h2 className="text-sm font-semibold">Template Sources</h2>
              <p className="text-[10px] text-ciab-text-muted">Sync templates from Git repositories</p>
            </div>
          </div>
          <button onClick={onClose} className="text-ciab-text-muted hover:text-ciab-text-primary transition-colors p-1">
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Sources list */}
        <div className="flex-1 overflow-auto p-4 space-y-3">
          {isLoading && (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-4 h-4 text-ciab-copper animate-spin" />
            </div>
          )}

          {!isLoading && (!sources || sources.length === 0) && !showAdd && (
            <div className="text-center py-8">
              <FolderGit2 className="w-10 h-10 text-ciab-text-muted/15 mx-auto mb-3" />
              <p className="text-sm text-ciab-text-secondary">No template sources</p>
              <p className="text-xs text-ciab-text-muted mt-1 max-w-xs mx-auto">
                Add a Git repository containing workspace templates as TOML files
              </p>
            </div>
          )}

          {sources?.map((source) => (
            <div key={source.id} className="card p-3.5 space-y-2">
              <div className="flex items-start justify-between">
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <GitBranch className="w-3.5 h-3.5 text-ciab-copper flex-shrink-0" />
                    <span className="text-sm font-medium text-ciab-text-primary truncate">{source.name}</span>
                  </div>
                  <p className="text-[10px] font-mono text-ciab-text-muted mt-0.5 truncate pl-5.5">{source.url}</p>
                </div>
                <div className="flex items-center gap-1 flex-shrink-0">
                  <button
                    onClick={() => handleSync(source.id)}
                    disabled={syncingId === source.id}
                    className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-copper hover:bg-ciab-copper/10 transition-colors disabled:opacity-40"
                    title="Sync now"
                  >
                    {syncingId === source.id ? (
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                    ) : (
                      <RefreshCw className="w-3.5 h-3.5" />
                    )}
                  </button>
                  <button
                    onClick={() => deleteSource.mutate(source.id)}
                    className="p-1.5 rounded-lg text-ciab-text-muted hover:text-state-failed hover:bg-state-failed/10 transition-colors"
                    title="Remove source"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
              <div className="flex items-center gap-3 pl-5.5 text-[10px] text-ciab-text-muted font-mono">
                <span className="flex items-center gap-1">
                  <GitBranch className="w-2.5 h-2.5" />
                  {source.branch}
                </span>
                <span>{source.template_count} templates</span>
                {source.last_synced_at && (
                  <span className="flex items-center gap-1">
                    <Clock className="w-2.5 h-2.5" />
                    synced {formatRelativeTime(source.last_synced_at)}
                  </span>
                )}
              </div>
            </div>
          ))}

          {/* Add form */}
          {showAdd && (
            <div className="rounded-xl border border-ciab-copper/20 bg-ciab-bg-secondary p-4 space-y-3 animate-fade-in">
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="label">Name</label>
                  <input
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    className="input w-full text-xs"
                    placeholder="My Templates"
                  />
                </div>
                <div>
                  <label className="label">Branch</label>
                  <input
                    type="text"
                    value={branch}
                    onChange={(e) => setBranch(e.target.value)}
                    className="input w-full text-xs font-mono"
                    placeholder="main"
                  />
                </div>
              </div>
              <div>
                <label className="label">Git URL</label>
                <input
                  type="text"
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  className="input w-full text-xs font-mono"
                  placeholder="https://github.com/org/ciab-templates.git"
                />
              </div>
              <div>
                <label className="label">Templates Path</label>
                <input
                  type="text"
                  value={templatesPath}
                  onChange={(e) => setTemplatesPath(e.target.value)}
                  className="input w-full text-xs font-mono"
                  placeholder=".ciab/templates"
                />
              </div>
              <div className="flex items-center justify-end gap-2">
                <button onClick={() => setShowAdd(false)} className="btn-ghost text-xs px-3 py-1.5">
                  Cancel
                </button>
                <button
                  onClick={handleAdd}
                  disabled={!name.trim() || !url.trim() || addSource.isPending}
                  className="btn-primary text-xs px-3 py-1.5 disabled:opacity-30"
                >
                  {addSource.isPending ? "Adding..." : "Add & Sync"}
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-4 border-t border-ciab-border">
          <button onClick={onClose} className="btn-ghost text-xs">
            Done
          </button>
          {!showAdd && (
            <button
              onClick={() => setShowAdd(true)}
              className="btn-secondary flex items-center gap-1.5 text-xs px-3 py-1.5"
            >
              <Plus className="w-3.5 h-3.5" />
              Add Source
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
