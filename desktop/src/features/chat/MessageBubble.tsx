import { useCallback, useRef, useState } from "react";
import type { Message, MessageContent } from "@/lib/api/types";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import {
  User,
  Wrench,
  Copy,
  Check,
  ChevronDown,
  ChevronRight,
  RotateCcw,
  Brain,
} from "lucide-react";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";
import ToolUseBlock from "./ToolUseBlock";

interface Props {
  message: Message;
  onRetry?: () => void;
}

export default function MessageBubble({ message, onRetry }: Props) {
  const isUser = message.role === "user";
  const isSystem = message.role === "system";
  const [copiedMessage, setCopiedMessage] = useState(false);

  const fullText = message.content
    .filter((c) => c.type === "text")
    .map((c) => (c as { type: "text"; text: string }).text)
    .join("\n");

  const handleCopyMessage = useCallback(() => {
    if (!fullText) return;
    navigator.clipboard.writeText(fullText).then(() => {
      setCopiedMessage(true);
      setTimeout(() => setCopiedMessage(false), 2000);
    });
  }, [fullText]);

  if (isSystem) {
    return (
      <div className="flex justify-center animate-fade-in">
        <span className="text-[10px] font-mono text-ciab-text-muted/60 bg-ciab-bg-elevated/50 px-3 py-1 rounded-full">
          {fullText}
        </span>
      </div>
    );
  }

  return (
    <div
      className={`group flex gap-3 animate-fade-in ${
        isUser ? "flex-row-reverse" : ""
      }`}
    >
      {/* Avatar */}
      <div
        className={`w-7 h-7 rounded-lg flex items-center justify-center flex-shrink-0 mt-1 ${
          isUser
            ? "bg-ciab-steel-blue/10 ring-1 ring-ciab-steel-blue/20"
            : "bg-ciab-copper/10 ring-1 ring-ciab-copper/20"
        }`}
      >
        {isUser ? (
          <User className="w-3.5 h-3.5 text-ciab-steel-blue" />
        ) : (
          <AgentProviderIcon provider="claude-code" size={14} />
        )}
      </div>

      {/* Content */}
      <div
        className={`flex-1 min-w-0 space-y-2 ${
          isUser ? "flex flex-col items-end" : ""
        }`}
      >
        {/* Role label + actions */}
        <div
          className={`flex items-center gap-2 ${
            isUser ? "flex-row-reverse" : ""
          }`}
        >
          <span
            className={`text-[10px] font-mono font-medium tracking-wide ${
              isUser ? "text-ciab-steel-blue/60" : "text-ciab-copper/60"
            }`}
          >
            {isUser ? "YOU" : "AGENT"}
          </span>

          {/* Message actions (show on hover) */}
          <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            {fullText && (
              <button
                onClick={handleCopyMessage}
                className="p-1 rounded text-ciab-text-muted/40 hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
                title="Copy message"
              >
                {copiedMessage ? (
                  <Check className="w-3 h-3 text-state-running" />
                ) : (
                  <Copy className="w-3 h-3" />
                )}
              </button>
            )}
            {onRetry && (
              <button
                onClick={onRetry}
                className="p-1 rounded text-ciab-text-muted/40 hover:text-ciab-text-secondary hover:bg-ciab-bg-hover transition-colors"
                title="Retry this message"
              >
                <RotateCcw className="w-3 h-3" />
              </button>
            )}
          </div>

          {/* Timestamp (hover only) */}
          <span className="text-[9px] font-mono text-ciab-text-muted/30 opacity-0 group-hover:opacity-100 transition-opacity">
            {new Date(message.timestamp).toLocaleTimeString()}
          </span>
        </div>

        {message.content.map((content, i) => (
          <ContentBlock key={i} content={content} isUser={isUser} />
        ))}
      </div>
    </div>
  );
}

function ContentBlock({
  content,
  isUser,
}: {
  content: MessageContent;
  isUser: boolean;
}) {
  switch (content.type) {
    case "text":
      if (!content.text?.trim()) return null;
      return (
        <div
          className={`rounded-xl px-4 py-3 max-w-[90%] ${
            isUser
              ? "bg-ciab-copper/8 border border-ciab-copper/15"
              : "bg-ciab-bg-card border border-ciab-border"
          }`}
        >
          <div
            className="text-sm prose prose-invert prose-sm max-w-none leading-relaxed
            prose-p:my-1.5 prose-p:leading-relaxed
            prose-code:text-ciab-copper-light prose-code:bg-ciab-bg-primary/80
            prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded-md prose-code:text-xs prose-code:font-mono
            prose-code:before:content-none prose-code:after:content-none
            prose-pre:bg-transparent prose-pre:p-0 prose-pre:my-3
            prose-a:text-ciab-steel-blue prose-a:no-underline hover:prose-a:underline
            prose-headings:text-ciab-text-primary prose-headings:font-semibold prose-headings:mt-4 prose-headings:mb-2
            prose-strong:text-ciab-text-primary prose-strong:font-semibold
            prose-ul:my-2 prose-ol:my-2 prose-li:my-0.5
            prose-blockquote:border-ciab-copper/30 prose-blockquote:text-ciab-text-secondary
            prose-hr:border-ciab-border
            prose-table:text-xs prose-th:text-ciab-text-secondary prose-td:text-ciab-text-secondary
            prose-th:border-ciab-border prose-td:border-ciab-border"
          >
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[rehypeHighlight]}
              components={{
                pre: ({ children }) => (
                  <CodeBlockWrapper>{children}</CodeBlockWrapper>
                ),
              }}
            >
              {content.text}
            </ReactMarkdown>
          </div>
        </div>
      );

    case "thinking":
      return <ThinkingContentBlock text={content.thinking} />;

    case "tool_use":
      return (
        <ToolUseBlock name={content.name} input={content.input} toolId={content.id} />
      );

    case "tool_result":
      return (
        <ToolResultBlock content={content.content} isError={content.is_error} />
      );

    case "image":
      return (
        <div className="rounded-xl border border-ciab-border overflow-hidden max-w-[80%] bg-ciab-bg-card">
          <img
            src={`data:${content.media_type};base64,${content.data}`}
            alt="Attached image"
            className="max-w-full max-h-[400px] object-contain"
            loading="lazy"
          />
        </div>
      );
  }
}

function ThinkingContentBlock({ text }: { text: string }) {
  const [expanded, setExpanded] = useState(false);
  const preview = text.length > 120 ? text.slice(0, 120) + "…" : text;

  return (
    <div className="max-w-[90%]">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg
          bg-violet-500/5 border border-violet-500/15
          hover:bg-violet-500/10 transition-colors w-full text-left group/think"
      >
        <Brain className="w-3 h-3 text-violet-400/70 flex-shrink-0" />
        <span className="text-[10px] font-mono font-medium text-violet-400/70 flex-shrink-0">
          Thinking
        </span>
        {!expanded && (
          <span className="text-[10px] text-ciab-text-muted/50 truncate flex-1 ml-1">
            {preview}
          </span>
        )}
        {expanded ? (
          <ChevronDown className="w-3 h-3 text-violet-400/50 flex-shrink-0" />
        ) : (
          <ChevronRight className="w-3 h-3 text-violet-400/50 flex-shrink-0" />
        )}
      </button>
      {expanded && (
        <div className="mt-1 px-3 py-2 rounded-lg bg-violet-500/5 border border-violet-500/10 max-h-[300px] overflow-y-auto">
          <pre className="text-[11px] font-mono text-ciab-text-secondary/80 whitespace-pre-wrap leading-relaxed">
            {text}
          </pre>
        </div>
      )}
    </div>
  );
}

function CodeBlockWrapper({ children }: { children: React.ReactNode }) {
  const [copied, setCopied] = useState(false);
  const preRef = useRef<HTMLDivElement>(null);

  const handleCopy = useCallback(() => {
    const codeEl = preRef.current?.querySelector("code");
    const text = codeEl?.textContent ?? "";
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, []);

  return (
    <div
      ref={preRef}
      className="relative group/code rounded-lg overflow-hidden border border-ciab-border bg-ciab-bg-primary my-3"
    >
      <div className="flex items-center justify-between px-3 py-1.5 bg-ciab-bg-elevated/50 border-b border-ciab-border">
        <span className="text-[10px] font-mono text-ciab-text-muted">code</span>
        <button
          onClick={handleCopy}
          className="flex items-center gap-1 text-[10px] font-mono text-ciab-text-muted
            hover:text-ciab-text-secondary transition-colors
            opacity-0 group-hover/code:opacity-100"
        >
          {copied ? (
            <>
              <Check className="w-3 h-3 text-state-running" />
              <span className="text-state-running">Copied</span>
            </>
          ) : (
            <>
              <Copy className="w-3 h-3" />
              <span>Copy</span>
            </>
          )}
        </button>
      </div>
      <div className="overflow-x-auto p-3 text-xs leading-relaxed [&_code]:bg-transparent [&_code]:p-0 [&_code]:border-0">
        {children}
      </div>
    </div>
  );
}

function ToolResultBlock({
  content,
  isError,
}: {
  content: string;
  isError?: boolean;
}) {
  const [expanded, setExpanded] = useState(false);
  const [copied, setCopied] = useState(false);
  const lines = content?.split("\n") ?? [];
  const isLong = lines.length > 8;
  const displayContent =
    isLong && !expanded ? lines.slice(0, 6).join("\n") + "\n..." : content;

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(content).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [content]);

  return (
    <div
      className={`rounded-xl border max-w-[90%] overflow-hidden group/result ${
        isError
          ? "border-state-failed/20 bg-state-failed/5"
          : "border-ciab-border bg-ciab-bg-card"
      }`}
    >
      <div className="px-3 py-1.5 border-b border-ciab-border/50 flex items-center gap-1.5">
        <Wrench className="w-3 h-3 text-ciab-text-muted" />
        <span
          className={`text-[10px] font-mono font-medium uppercase tracking-wide flex-1 ${
            isError ? "text-state-failed" : "text-ciab-text-muted"
          }`}
        >
          {isError ? "Error" : "Result"}
        </span>
        <button
          onClick={handleCopy}
          className="p-0.5 rounded opacity-0 group-hover/result:opacity-100 transition-opacity
            text-ciab-text-muted hover:text-ciab-text-secondary"
          title="Copy result"
        >
          {copied ? (
            <Check className="w-3 h-3 text-state-running" />
          ) : (
            <Copy className="w-3 h-3" />
          )}
        </button>
      </div>
      <pre className="text-[11px] font-mono p-3 whitespace-pre-wrap overflow-x-auto text-ciab-text-secondary leading-relaxed">
        {displayContent}
      </pre>
      {isLong && (
        <button
          onClick={() => setExpanded(!expanded)}
          className="w-full px-3 py-1.5 border-t border-ciab-border/50 flex items-center justify-center gap-1
            text-[10px] font-mono text-ciab-text-muted hover:text-ciab-text-secondary transition-colors"
        >
          {expanded ? (
            <>
              <ChevronDown className="w-3 h-3" />
              Show less
            </>
          ) : (
            <>
              <ChevronRight className="w-3 h-3" />
              Show all {lines.length} lines
            </>
          )}
        </button>
      )}
    </div>
  );
}
