import { useState, useRef, useEffect, useCallback, useMemo } from "react";
import {
  ArrowUp,
  Square,
  Paperclip,
  Image as ImageIcon,
  FileText,
  FolderOpen,
  X,
} from "lucide-react";
import { useInterruptSession } from "@/lib/hooks/use-sessions";
import { useSlashCommands } from "@/lib/hooks/use-slash-commands";
import { useFilePicker } from "@/lib/hooks/use-directory-picker";
import SlashCommandPopup from "./SlashCommandPopup";
import type { SlashCommand } from "@/lib/api/types";

export interface ImageAttachment {
  data: string; // base64
  media_type: string;
  preview: string; // data URI for preview
  name: string;
}

export interface FileAttachment {
  content: string;
  name: string;
  size: number;
}

interface Props {
  onSend: (text: string, images?: ImageAttachment[]) => void;
  disabled: boolean;
  isProcessing?: boolean;
  sessionId: string;
  agentProvider?: string;
}

export default function ChatInput({ onSend, disabled, isProcessing = false, sessionId, agentProvider }: Props) {
  const [text, setText] = useState("");
  const [images, setImages] = useState<ImageAttachment[]>([]);
  const [files, setFiles] = useState<FileAttachment[]>([]);
  const [isDragOver, setIsDragOver] = useState(false);
  const [showAttachMenu, setShowAttachMenu] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const textFileInputRef = useRef<HTMLInputElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const interrupt = useInterruptSession(sessionId);
  const { pickFile } = useFilePicker();

  // Slash command state
  const [showSlashMenu, setShowSlashMenu] = useState(false);
  const [slashFilter, setSlashFilter] = useState("");
  const [slashSelectedIndex, setSlashSelectedIndex] = useState(0);
  const { data: slashCommands } = useSlashCommands(agentProvider);

  const filteredSlashCommands = useMemo(() => {
    if (!slashCommands || !showSlashMenu) return [];
    if (!slashFilter) return slashCommands;
    const lower = slashFilter.toLowerCase();
    return slashCommands.filter((cmd) => cmd.name.toLowerCase().startsWith(lower));
  }, [slashCommands, slashFilter, showSlashMenu]);

  // Auto-resize textarea
  useEffect(() => {
    const el = textareaRef.current;
    if (el) {
      el.style.height = "auto";
      el.style.height = `${Math.min(el.scrollHeight, 200)}px`;
    }
  }, [text]);

  // Focus textarea on mount and when not disabled
  useEffect(() => {
    if (!disabled) {
      textareaRef.current?.focus();
    }
  }, [disabled]);

  const processFile = useCallback((file: File) => {
    if (file.type.startsWith("image/")) {
      if (file.size > 20 * 1024 * 1024) return; // 20MB limit
      const reader = new FileReader();
      reader.onload = () => {
        const dataUrl = reader.result as string;
        const base64 = dataUrl.split(",")[1];
        setImages((prev) => [
          ...prev,
          {
            data: base64,
            media_type: file.type,
            preview: dataUrl,
            name: file.name,
          },
        ]);
      };
      reader.readAsDataURL(file);
    } else {
      // Text/code files — read as text and include as context
      if (file.size > 1 * 1024 * 1024) return; // 1MB limit for text
      const reader = new FileReader();
      reader.onload = () => {
        const content = reader.result as string;
        setFiles((prev) => [
          ...prev,
          { content, name: file.name, size: file.size },
        ]);
      };
      reader.readAsText(file);
    }
  }, []);

  const removeFile = useCallback((index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  }, []);

  // Handle paste (images from clipboard)
  const handlePaste = useCallback(
    (e: React.ClipboardEvent) => {
      const items = e.clipboardData?.items;
      if (!items) return;

      for (const item of items) {
        if (item.type.startsWith("image/")) {
          e.preventDefault();
          const file = item.getAsFile();
          if (file) processFile(file);
        }
      }
    },
    [processFile]
  );

  // Handle drag & drop
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragOver(false);

      const files = e.dataTransfer?.files;
      if (files) {
        for (const file of files) {
          processFile(file);
        }
      }
    },
    [processFile]
  );

  const removeImage = useCallback((index: number) => {
    setImages((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const handleSend = useCallback(() => {
    if ((!text.trim() && images.length === 0 && files.length === 0) || disabled) return;
    // Prepend file contents as context before the user's message
    let messageText = text.trim();
    if (files.length > 0) {
      const fileContext = files
        .map((f) => `<file name="${f.name}">\n${f.content}\n</file>`)
        .join("\n\n");
      messageText = messageText
        ? `${fileContext}\n\n${messageText}`
        : fileContext;
    }
    onSend(messageText, images.length > 0 ? images : undefined);
    setText("");
    setImages([]);
    setFiles([]);
    setShowSlashMenu(false);
    setShowAttachMenu(false);
  }, [text, images, files, disabled, onSend]);

  const handleSlashSelect = useCallback(
    (cmd: SlashCommand) => {
      setShowSlashMenu(false);
      if (cmd.args.length === 0) {
        // No args — auto-submit
        onSend(`/${cmd.name}`);
        setText("");
        setImages([]);
      } else {
        // Has args — insert with trailing space, let user type args
        setText(`/${cmd.name} `);
        // Refocus textarea
        setTimeout(() => textareaRef.current?.focus(), 0);
      }
    },
    [onSend]
  );

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const val = e.target.value;
      setText(val);

      // Show slash menu when input starts with "/" on a single line
      if (val.startsWith("/") && !val.includes("\n")) {
        const filterText = val.slice(1).split(" ")[0]; // only filter by first word after /
        // If user has typed a space after the command name, hide menu
        if (val.includes(" ") && val.indexOf(" ") > 1) {
          setShowSlashMenu(false);
        } else {
          setShowSlashMenu(true);
          setSlashFilter(filterText);
          setSlashSelectedIndex(0);
        }
      } else {
        setShowSlashMenu(false);
      }
    },
    []
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Slash command menu navigation
      if (showSlashMenu && filteredSlashCommands.length > 0) {
        if (e.key === "ArrowUp") {
          e.preventDefault();
          setSlashSelectedIndex((i) => Math.max(0, i - 1));
          return;
        }
        if (e.key === "ArrowDown") {
          e.preventDefault();
          setSlashSelectedIndex((i) =>
            Math.min(filteredSlashCommands.length - 1, i + 1)
          );
          return;
        }
        if (e.key === "Enter" || e.key === "Tab") {
          e.preventDefault();
          handleSlashSelect(filteredSlashCommands[slashSelectedIndex]);
          return;
        }
        if (e.key === "Escape") {
          e.preventDefault();
          setShowSlashMenu(false);
          return;
        }
      }

      // Enter to send, Shift+Enter for newline
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
      // Escape to clear
      if (e.key === "Escape") {
        if (showAttachMenu) {
          setShowAttachMenu(false);
        } else if (images.length > 0 || files.length > 0) {
          setImages([]);
          setFiles([]);
        } else if (text) {
          setText("");
        }
      }
    },
    [handleSend, images.length, files.length, text, showSlashMenu, showAttachMenu, filteredSlashCommands, slashSelectedIndex, handleSlashSelect]
  );

  // Close attach menu when clicking outside
  useEffect(() => {
    if (!showAttachMenu) return;
    const handler = (e: MouseEvent) => {
      if (wrapperRef.current && !wrapperRef.current.contains(e.target as Node)) {
        setShowAttachMenu(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [showAttachMenu]);

  // Global keyboard shortcut: Cmd+K to focus input
  useEffect(() => {
    const handleGlobalKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        textareaRef.current?.focus();
      }
      // Cmd+. or Ctrl+C to interrupt when processing
      if (disabled && ((e.metaKey && e.key === ".") || (e.ctrlKey && e.key === "c"))) {
        e.preventDefault();
        interrupt.mutate();
      }
    };
    window.addEventListener("keydown", handleGlobalKey);
    return () => window.removeEventListener("keydown", handleGlobalKey);
  }, [disabled, interrupt]);

  const canSend = (text.trim() || images.length > 0 || files.length > 0) && !disabled;

  return (
    <div
      ref={wrapperRef}
      className={`relative border-t border-ciab-border transition-colors ${
        isDragOver ? "bg-ciab-copper/5 border-ciab-copper/30" : ""
      }`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Drag overlay */}
      {isDragOver && (
        <div className="absolute inset-0 z-10 flex items-center justify-center bg-ciab-bg-primary/80 backdrop-blur-sm border-2 border-dashed border-ciab-copper/40 rounded-lg m-1">
          <div className="flex flex-col items-center gap-2 text-ciab-copper">
            <Paperclip className="w-8 h-8" />
            <span className="text-sm font-medium">Drop files here</span>
          </div>
        </div>
      )}

      {/* Attachment previews */}
      {(images.length > 0 || files.length > 0) && (
        <div className="flex gap-2 px-3 pt-3 pb-1 overflow-x-auto">
          {images.map((img, i) => (
            <div
              key={`img-${i}`}
              className="relative group flex-shrink-0 w-16 h-16 rounded-lg overflow-hidden border border-ciab-border bg-ciab-bg-elevated"
            >
              <img
                src={img.preview}
                alt={img.name}
                className="w-full h-full object-cover"
              />
              <button
                onClick={() => removeImage(i)}
                className="absolute -top-1 -right-1 w-5 h-5 rounded-full bg-ciab-bg-primary border border-ciab-border
                  flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity
                  hover:bg-state-failed/20 hover:border-state-failed/30"
              >
                <X className="w-3 h-3 text-ciab-text-secondary" />
              </button>
              <div className="absolute bottom-0 left-0 right-0 bg-black/60 px-1 py-0.5">
                <span className="text-[8px] text-white/80 font-mono truncate block">
                  {img.name}
                </span>
              </div>
            </div>
          ))}
          {files.map((file, i) => (
            <div
              key={`file-${i}`}
              className="relative group flex-shrink-0 w-28 h-16 rounded-lg border border-ciab-border bg-ciab-bg-elevated flex flex-col items-center justify-center px-2"
            >
              <FileText className="w-4 h-4 text-ciab-steel-blue mb-0.5" />
              <span className="text-[9px] text-ciab-text-secondary font-mono truncate w-full text-center">
                {file.name}
              </span>
              <span className="text-[8px] text-ciab-text-muted">
                {file.size < 1024
                  ? `${file.size}B`
                  : `${(file.size / 1024).toFixed(1)}KB`}
              </span>
              <button
                onClick={() => removeFile(i)}
                className="absolute -top-1 -right-1 w-5 h-5 rounded-full bg-ciab-bg-primary border border-ciab-border
                  flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity
                  hover:bg-state-failed/20 hover:border-state-failed/30"
              >
                <X className="w-3 h-3 text-ciab-text-secondary" />
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Slash command popup — positioned above the entire input area */}
      {showSlashMenu && slashCommands && slashCommands.length > 0 && (
        <SlashCommandPopup
          commands={slashCommands}
          filter={slashFilter}
          selectedIndex={slashSelectedIndex}
          onSelect={handleSlashSelect}
          onClose={() => setShowSlashMenu(false)}
        />
      )}

      {/* Input area */}
      <div className="flex items-end gap-2 p-3">
        {/* Attach button with dropdown */}
        <div className="relative flex items-center gap-1 pb-1">
          <button
            onClick={() => setShowAttachMenu(!showAttachMenu)}
            disabled={disabled}
            className={`p-1.5 rounded-md transition-colors disabled:opacity-30 disabled:cursor-not-allowed ${
              showAttachMenu
                ? "text-ciab-copper bg-ciab-copper/10"
                : "text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover"
            }`}
            title="Attach files"
          >
            <Paperclip className="w-4 h-4" />
          </button>

          {/* Attach dropdown */}
          {showAttachMenu && (
            <div className="absolute bottom-full left-0 mb-1 w-44 bg-ciab-bg-card border border-ciab-border rounded-lg shadow-lg z-20 animate-fade-in overflow-hidden">
              <button
                onClick={() => {
                  fileInputRef.current?.click();
                  setShowAttachMenu(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
              >
                <ImageIcon className="w-3.5 h-3.5" />
                Attach Image
              </button>
              <button
                onClick={() => {
                  textFileInputRef.current?.click();
                  setShowAttachMenu(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
              >
                <FileText className="w-3.5 h-3.5" />
                Attach File
              </button>
              <button
                onClick={async () => {
                  setShowAttachMenu(false);
                  const result = await pickFile({ multiple: true });
                  if (result) {
                    // In Tauri, pickFile returns paths — read them via shell
                    const paths = Array.isArray(result) ? result : [result];
                    for (const path of paths) {
                      try {
                        const { Command } = await import("@tauri-apps/plugin-shell");
                        const output = await Command.create("cat", [path]).execute();
                        if (output.code === 0) {
                          const name = path.split("/").pop() || "file";
                          setFiles((prev) => [
                            ...prev,
                            { content: output.stdout, name, size: output.stdout.length },
                          ]);
                        }
                      } catch {
                        // Ignore read errors
                      }
                    }
                  }
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors border-t border-ciab-border"
              >
                <FolderOpen className="w-3.5 h-3.5" />
                Browse Files...
              </button>
            </div>
          )}

          <input
            ref={fileInputRef}
            type="file"
            accept="image/*"
            multiple
            className="hidden"
            onChange={(e) => {
              const inputFiles = e.target.files;
              if (inputFiles) {
                for (const file of inputFiles) processFile(file);
              }
              e.target.value = "";
            }}
          />
          <input
            ref={textFileInputRef}
            type="file"
            accept=".txt,.md,.json,.yaml,.yml,.toml,.ts,.tsx,.js,.jsx,.py,.rs,.go,.html,.css,.sh,.bash,.zsh,.env,.log,.csv,.xml,.sql,.rb,.php,.java,.c,.cpp,.h,.hpp,.swift,.kt"
            multiple
            className="hidden"
            onChange={(e) => {
              const inputFiles = e.target.files;
              if (inputFiles) {
                for (const file of inputFiles) processFile(file);
              }
              e.target.value = "";
            }}
          />
        </div>

        {/* Textarea */}
        <div className="flex-1 relative">
          <textarea
            ref={textareaRef}
            value={text}
            onChange={handleChange}
            onKeyDown={handleKeyDown}
            onPaste={handlePaste}
            placeholder={
              disabled
                ? "Agent is working..."
                : isProcessing
                  ? "Type to queue a message... (type / for commands)"
                  : "Message the agent... (type / for commands)"
            }
            rows={1}
            className={`w-full bg-ciab-bg-secondary border rounded-xl px-4 py-2.5
              text-sm text-ciab-text-primary placeholder:text-ciab-text-muted/50
              resize-none min-h-[40px] max-h-[200px]
              focus:outline-none focus:ring-1
              disabled:opacity-50 disabled:cursor-not-allowed
              transition-all ${
                showSlashMenu
                  ? "border-ciab-copper/40 ring-1 ring-ciab-copper/20"
                  : "border-ciab-border focus:border-ciab-copper/40 focus:ring-ciab-copper/20"
              }`}
            disabled={disabled}
          />
        </div>

        {/* Send / Stop buttons */}
        <div className="pb-1 flex items-center gap-1">
          {isProcessing && (
            <button
              onClick={() => interrupt.mutate()}
              className="w-9 h-9 rounded-xl bg-state-failed/10 hover:bg-state-failed/20 text-state-failed
                flex items-center justify-center transition-all hover:scale-105 active:scale-95"
              title="Stop (⌘.)"
            >
              <Square className="w-4 h-4" fill="currentColor" />
            </button>
          )}
          <button
            onClick={handleSend}
            disabled={!canSend}
            className={`w-9 h-9 rounded-xl flex items-center justify-center transition-all
              ${
                canSend
                  ? isProcessing
                    ? "bg-amber-600 hover:bg-amber-500 text-white hover:scale-105 active:scale-95 shadow-lg shadow-amber-600/20"
                    : "bg-ciab-copper hover:bg-ciab-copper-dark text-white hover:scale-105 active:scale-95 shadow-lg shadow-ciab-copper/20"
                  : "bg-ciab-bg-elevated text-ciab-text-muted cursor-not-allowed"
              }`}
            title={isProcessing ? "Queue message (Enter)" : "Send (Enter)"}
          >
            <ArrowUp className="w-4 h-4" strokeWidth={2.5} />
          </button>
        </div>
      </div>

      {/* Hint bar */}
      <div className="flex items-center justify-between px-4 pb-2 text-[10px] text-ciab-text-muted/60 font-mono">
        <div className="flex items-center gap-3">
          <span>
            <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated text-ciab-text-muted text-[9px]">
              Enter
            </kbd>{" "}
            send
          </span>
          <span>
            <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated text-ciab-text-muted text-[9px]">
              Shift+Enter
            </kbd>{" "}
            newline
          </span>
          {disabled && (
            <span>
              <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated text-ciab-text-muted text-[9px]">
                {"⌘."}
              </kbd>{" "}
              stop
            </span>
          )}
        </div>
        <div className="flex items-center gap-3">
          <span>
            <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated text-ciab-text-muted text-[9px]">
              /
            </kbd>{" "}
            commands
          </span>
          <span>
            <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated text-ciab-text-muted text-[9px]">
              {"⌘K"}
            </kbd>{" "}
            focus
          </span>
          <span>
            <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated text-ciab-text-muted text-[9px]">
              {"⌘V"}
            </kbd>{" "}
            paste
          </span>
        </div>
      </div>
    </div>
  );
}
