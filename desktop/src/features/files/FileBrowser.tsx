import { useState, useMemo } from "react";
import { useFileList } from "@/lib/hooks/use-files";
import {
  Folder,
  File,
  FileText,
  FileCode,
  FileJson,
  Image,
  ChevronRight,
  ArrowUp,
  RefreshCw,
  Search,
  X,
  Home,
} from "lucide-react";
import { formatBytes, formatRelativeTime } from "@/lib/utils/format";
import LoadingSpinner from "@/components/shared/LoadingSpinner";

interface Props {
  sandboxId: string;
}

const FILE_ICONS: Record<string, typeof File> = {
  ts: FileCode,
  tsx: FileCode,
  js: FileCode,
  jsx: FileCode,
  py: FileCode,
  rs: FileCode,
  go: FileCode,
  rb: FileCode,
  java: FileCode,
  c: FileCode,
  cpp: FileCode,
  h: FileCode,
  json: FileJson,
  yaml: FileText,
  yml: FileText,
  toml: FileText,
  md: FileText,
  txt: FileText,
  png: Image,
  jpg: Image,
  jpeg: Image,
  gif: Image,
  svg: Image,
  webp: Image,
};

function getFileIcon(name: string, isDir: boolean) {
  if (isDir) return Folder;
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  return FILE_ICONS[ext] ?? File;
}

export default function FileBrowser({ sandboxId }: Props) {
  const [currentPath, setCurrentPath] = useState("/workspace");
  const [filterText, setFilterText] = useState("");
  const { data: fileList, isLoading, refetch } = useFileList(sandboxId, currentPath);

  const navigateTo = (path: string) => {
    setCurrentPath(path);
    setFilterText("");
  };

  const navigateUp = () => {
    const parent = currentPath.split("/").slice(0, -1).join("/") || "/";
    setCurrentPath(parent);
    setFilterText("");
  };

  const pathSegments = currentPath.split("/").filter(Boolean);

  const sortedFiles = useMemo(() => {
    let files = [...(fileList ?? [])].sort((a, b) => {
      if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
      return a.path.localeCompare(b.path);
    });

    if (filterText) {
      const lower = filterText.toLowerCase();
      files = files.filter((f) => {
        const name = f.path.split("/").pop() ?? f.path;
        return name.toLowerCase().includes(lower);
      });
    }

    return files;
  }, [fileList, filterText]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-2 flex-shrink-0">
        <div className="flex items-center gap-1.5">
          <button
            onClick={() => navigateTo("/workspace")}
            className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
            title="Home (/workspace)"
          >
            <Home className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={navigateUp}
            className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors disabled:opacity-30"
            disabled={currentPath === "/"}
            title="Go up"
          >
            <ArrowUp className="w-3.5 h-3.5" />
          </button>

          {/* Breadcrumb */}
          <div className="flex items-center gap-0.5 text-xs font-mono ml-1 overflow-x-auto">
            <button
              onClick={() => navigateTo("/")}
              className="text-ciab-text-muted hover:text-ciab-copper transition-colors flex-shrink-0"
            >
              /
            </button>
            {pathSegments.map((segment, i) => (
              <span key={i} className="flex items-center gap-0.5 flex-shrink-0">
                <ChevronRight className="w-2.5 h-2.5 text-ciab-text-muted/30" />
                <button
                  onClick={() =>
                    navigateTo("/" + pathSegments.slice(0, i + 1).join("/"))
                  }
                  className={`transition-colors ${
                    i === pathSegments.length - 1
                      ? "text-ciab-text-primary font-medium"
                      : "text-ciab-text-muted hover:text-ciab-copper"
                  }`}
                >
                  {segment}
                </button>
              </span>
            ))}
          </div>
        </div>

        <div className="flex items-center gap-1">
          <span className="text-[9px] font-mono text-ciab-text-muted/50 mr-1">
            {sortedFiles.length} items
          </span>
          <button
            onClick={() => refetch()}
            className="p-1.5 rounded-lg text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
            title="Refresh"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Search */}
      <div className="mb-2 flex-shrink-0">
        <div className="flex items-center gap-2 bg-ciab-bg-secondary border border-ciab-border rounded-lg px-3 py-1.5">
          <Search className="w-3 h-3 text-ciab-text-muted" />
          <input
            type="text"
            value={filterText}
            onChange={(e) => setFilterText(e.target.value)}
            placeholder="Filter files..."
            className="flex-1 bg-transparent border-none outline-none text-xs font-mono text-ciab-text-primary placeholder:text-ciab-text-muted/40"
          />
          {filterText && (
            <button
              onClick={() => setFilterText("")}
              className="text-ciab-text-muted hover:text-ciab-text-secondary"
            >
              <X className="w-3 h-3" />
            </button>
          )}
        </div>
      </div>

      {/* File list */}
      <div className="flex-1 bg-ciab-bg-card rounded-xl border border-ciab-border overflow-y-auto min-h-0">
        <table className="w-full">
          <thead className="sticky top-0 bg-ciab-bg-card z-10">
            <tr className="border-b border-ciab-border">
              <th className="text-left px-4 py-2.5 text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider font-medium">
                Name
              </th>
              <th className="text-right px-4 py-2.5 text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider w-24 font-medium">
                Size
              </th>
              <th className="text-right px-4 py-2.5 text-[10px] font-mono text-ciab-text-muted uppercase tracking-wider w-28 font-medium">
                Modified
              </th>
            </tr>
          </thead>
          <tbody>
            {sortedFiles.map((file) => {
              const name = file.path.split("/").pop() ?? file.path;
              const FileIcon = getFileIcon(name, file.is_dir);
              return (
                <tr
                  key={file.path}
                  className={`border-b border-ciab-border/20 last:border-0 hover:bg-ciab-bg-hover/30 transition-colors ${
                    file.is_dir ? "cursor-pointer" : ""
                  }`}
                  onClick={() => file.is_dir && navigateTo(file.path)}
                >
                  <td className="px-4 py-2">
                    <div className="flex items-center gap-2.5">
                      <FileIcon
                        className={`w-4 h-4 flex-shrink-0 ${
                          file.is_dir
                            ? "text-ciab-copper/70"
                            : "text-ciab-text-muted/50"
                        }`}
                      />
                      <span
                        className={`text-xs font-mono ${
                          file.is_dir
                            ? "text-ciab-text-primary font-medium"
                            : "text-ciab-text-secondary"
                        }`}
                      >
                        {name}
                        {file.is_dir && (
                          <span className="text-ciab-text-muted/30">/</span>
                        )}
                      </span>
                    </div>
                  </td>
                  <td className="px-4 py-2 text-right text-[10px] text-ciab-text-muted font-mono tabular-nums">
                    {file.is_dir ? "\u2014" : formatBytes(file.size)}
                  </td>
                  <td className="px-4 py-2 text-right text-[10px] text-ciab-text-muted font-mono">
                    {file.modified_at
                      ? formatRelativeTime(file.modified_at)
                      : "\u2014"}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>

        {sortedFiles.length === 0 && (
          <div className="text-center py-12 text-ciab-text-muted text-xs">
            {filterText ? (
              <span>
                No files matching &ldquo;
                <span className="text-ciab-copper">{filterText}</span>&rdquo;
              </span>
            ) : (
              "Empty directory"
            )}
          </div>
        )}
      </div>
    </div>
  );
}
