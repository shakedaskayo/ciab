import { useState, useRef, useEffect, useCallback, useMemo } from "react";
import {
  useSessions,
  useCreateSession,
  useSendMessage,
  useSetPermissionMode,
  useRespondToPermission,
  useRespondToUserInput,
  useUpdateSessionSkills,
  type SendMessagePayload,
  type SessionSkill,
} from "@/lib/hooks/use-sessions";
import { useSession } from "@/lib/hooks/use-sessions";
import { useSessionStream } from "@/lib/hooks/use-stream";

import type {
  StreamEvent,
  Message,
  Session,
  PermissionMode,
  PermissionRequestData,
  UserInputRequestData,
} from "@/lib/api/types";
import { sessions } from "@/lib/api/endpoints";
import type { ImageAttachment } from "./ChatInput";
import MessageBubble from "./MessageBubble";
import ChatInput from "./ChatInput";
import PermissionConfirmation from "./PermissionConfirmation";
import PermissionModeSelector from "./PermissionModeSelector";
import UserInputRequest from "./UserInputRequest";
import SkillPicker from "./SkillPicker";
import ActivityPanel from "./ActivityPanel";
import type { FileActivity } from "./ActivityPanel";
import TodoListBlock, { parseTodoInput } from "./TodoListBlock";
import ToolUseBlock from "./ToolUseBlock";
import AgentProviderIcon from "@/components/shared/AgentProviderIcon";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import {
  MessageSquare,
  Plus,
  Clock,
  DollarSign,
  Zap,
  ArrowDown,
  Sparkles,
  Terminal,
  FileEdit,
  Search,
  GitBranch,
  User,
  Brain,
  ChevronDown,
  ChevronRight,
  Bot,
  AlertTriangle,
  PanelRightOpen,
  PanelRightClose,
} from "lucide-react";
import { formatRelativeTime } from "@/lib/utils/format";

interface Props {
  sandboxId: string;
  agentProvider?: string;
}

// Optimistic user message shown before server confirms
interface OptimisticMessage {
  text: string;
  images?: Array<{ data: string; media_type: string }>;
  timestamp: Date;
}

export default function ChatView({ sandboxId, agentProvider }: Props) {
  const { data: sessionList, refetch: refetchSessions } = useSessions(sandboxId);
  const createSession = useCreateSession(sandboxId);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [streamingText, setStreamingText] = useState("");
  const [thinkingText, setThinkingText] = useState("");
  const [activeSubagent, setActiveSubagent] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const [lastCost, setLastCost] = useState<number | null>(null);
  const [lastDuration, setLastDuration] = useState<number | null>(null);
  const [activeTool, setActiveTool] = useState<string | null>(null);
  const [showScrollButton, setShowScrollButton] = useState(false);
  const [totalCost, setTotalCost] = useState(0);
  const [optimisticMsg, setOptimisticMsg] = useState<OptimisticMessage | null>(null);
  const [permissionMode, setPermissionModeLocal] = useState<PermissionMode>("auto_approve");
  const [pendingPermissions, setPendingPermissions] = useState<
    Map<string, PermissionRequestData & { status?: "approved" | "denied" }>
  >(new Map());
  const [pendingUserQuestions, setPendingUserQuestions] = useState<UserInputRequestData | null>(null);
  const [toolProgressText, setToolProgressText] = useState<string | null>(null);
  const [resultError, setResultError] = useState<{ error_type: string; message: string; cost_usd?: number } | null>(null);
  const [messageQueue, setMessageQueue] = useState<Array<{ id: string; prompt_text: string; queued_at: string }>>([]);
  const [fileActivities, setFileActivities] = useState<FileActivity[]>([]);
  const [liveTodos, setLiveTodos] = useState<Array<{ id: string; content: string; status: "pending" | "in_progress" | "completed"; priority?: "high" | "medium" | "low" }>>([]);
  const [streamingToolInput, setStreamingToolInput] = useState("");
  const activeToolRef = useRef<string | null>(null);
  const [completedToolCalls, setCompletedToolCalls] = useState<
    Array<{ id: string; name: string; input: unknown; agentName?: string }>
  >([]);
  const activeSubagentRef = useRef<string | null>(null);
  const activeToolMetaRef = useRef<{ id: string; agentName: string | null } | null>(null);
  const [showActivityPanel, setShowActivityPanel] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);
  const isNearBottomRef = useRef(true);

  // Sort sessions chronologically (oldest first = tab 1)
  const sortedSessions = useMemo(() => {
    if (!sessionList) return [];
    return [...sessionList].sort(
      (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
    );
  }, [sessionList]);

  // Auto-select first session
  useEffect(() => {
    if (sortedSessions.length > 0 && !activeSessionId) {
      setActiveSessionId(sortedSessions[0].id);
    }
  }, [sortedSessions, activeSessionId]);

  const { data: session, refetch: refetchSession } = useSession(
    activeSessionId ?? ""
  );
  const sendMessage = useSendMessage(activeSessionId ?? "");
  const setPermissionModeApi = useSetPermissionMode(activeSessionId ?? "");
  const respondToPermission = useRespondToPermission(activeSessionId ?? "");
  const respondToUserInput = useRespondToUserInput(activeSessionId ?? "");
  const updateSessionSkills = useUpdateSessionSkills(activeSessionId ?? "");
  const [showSkillPicker, setShowSkillPicker] = useState(false);

  // Permission mode change handler
  const handlePermissionModeChange = useCallback(
    (newMode: PermissionMode) => {
      setPermissionModeLocal(newMode);
      if (activeSessionId) {
        setPermissionModeApi.mutate({ mode: newMode });
      }
    },
    [activeSessionId, setPermissionModeApi]
  );

  // Permission approval/denial handlers
  const handlePermissionApprove = useCallback(
    (requestId: string) => {
      setPendingPermissions((prev) => {
        const next = new Map(prev);
        const entry = next.get(requestId);
        if (entry) next.set(requestId, { ...entry, status: "approved" });
        return next;
      });
      respondToPermission.mutate({ requestId, approved: true });
    },
    [respondToPermission]
  );

  const handlePermissionDeny = useCallback(
    (requestId: string) => {
      setPendingPermissions((prev) => {
        const next = new Map(prev);
        const entry = next.get(requestId);
        if (entry) next.set(requestId, { ...entry, status: "denied" });
        return next;
      });
      respondToPermission.mutate({ requestId, approved: false });
    },
    [respondToPermission]
  );

  const handleAllowTool = useCallback(
    (toolName: string) => {
      if (activeSessionId) {
        // Read current always_allow and append this tool
        setPermissionModeApi.mutate({
          mode: permissionMode,
          always_allow: [toolName],
        });
      }
    },
    [activeSessionId, permissionMode, setPermissionModeApi]
  );

  const handleSwitchMode = useCallback(
    (mode: PermissionMode) => {
      handlePermissionModeChange(mode);
    },
    [handlePermissionModeChange]
  );

  // Track message count and clear optimistic message when the server
  // confirms the user message is persisted (message count increased).
  const prevMsgCount = useRef(0);
  useEffect(() => {
    if (session?.messages) {
      const count = session.messages.length;
      // Clear optimistic msg only when server messages actually grew
      // (i.e. the user message we sent is now in the DB).
      if (optimisticMsg && count > prevMsgCount.current) {
        setOptimisticMsg(null);
      }
      prevMsgCount.current = count;
    }
  }, [session?.messages, optimisticMsg]);

  const handleStreamEvent = useCallback(
    (event: StreamEvent) => {
      switch (event.event_type) {
        case "text_delta": {
          const data = event.data as { text?: string };
          if (data.text) {
            setStreamingText((prev) => prev + data.text);
            setThinkingText("");
            setIsProcessing(true);
            setActiveTool(null);
            setOptimisticMsg(null);
          }
          break;
        }
        case "thinking_delta": {
          const data = event.data as { text?: string };
          if (data.text) {
            setThinkingText((prev) => prev + data.text);
            setIsProcessing(true);
            setOptimisticMsg(null);
          }
          break;
        }
        case "text_complete": {
          const data = event.data as {
            cost_usd?: number;
            duration_ms?: number;
          };
          if (data.cost_usd) {
            setLastCost(data.cost_usd);
            setTotalCost((prev) => prev + data.cost_usd!);
          }
          if (data.duration_ms) setLastDuration(data.duration_ms);
          break;
        }
        case "tool_use_start": {
          const data = event.data as { id?: string; name?: string; input?: Record<string, unknown>; streaming?: boolean };
          const toolName = data.name ?? "tool";
          setActiveTool(toolName);
          activeToolRef.current = toolName;
          setThinkingText("");
          setIsProcessing(true);
          setOptimisticMsg(null);
          // Reset streaming tool input for the new tool
          setStreamingToolInput("");
          // Store current tool metadata for accumulation on completion
          activeToolMetaRef.current = { id: data.id ?? event.id, agentName: activeSubagentRef.current };

          // Track TodoWrite updates in real-time (from complete input)
          if (data.name === "TodoWrite" && data.input && !data.streaming) {
            const parsed = parseTodoInput(data.input);
            if (parsed) setLiveTodos(parsed);
          }

          // Extract file activity from tool_use_start for the activity panel
          if (data.name && data.input) {
            const fp = (data.input.file_path ?? data.input.path ?? data.input.command) as string | undefined;
            if (fp) {
              const action = data.name === "Edit" || data.name === "MultiEdit" ? "edited"
                : data.name === "Write" || data.name === "NotebookEdit" ? "written"
                : data.name === "Read" ? "read"
                : data.name === "Bash" ? "executed"
                : data.name === "Grep" ? "searched"
                : data.name === "Glob" ? "listed"
                : "accessed";
              setFileActivities((prev) => [
                ...prev,
                {
                  id: event.id,
                  tool_name: data.name!,
                  file_path: typeof fp === "string" && fp.length > 80 ? fp.slice(0, 77) + "..." : (fp as string),
                  action,
                  timestamp: new Date(),
                },
              ]);
            }
          }
          break;
        }
        case "tool_input_delta": {
          const data = event.data as { partial_json?: string };
          if (data.partial_json) {
            setStreamingToolInput((prev) => {
              const updated = prev + data.partial_json;
              // Try to parse accumulated JSON for TodoWrite live updates
              if (activeToolRef.current === "TodoWrite") {
                try {
                  const parsed = JSON.parse(updated);
                  const todos = parseTodoInput(parsed);
                  if (todos) setLiveTodos(todos);
                } catch {
                  // Incomplete JSON — will parse when more chunks arrive
                }
              }
              return updated;
            });
          }
          break;
        }
        case "tool_use_complete":
        case "tool_result": {
          // Accumulate the completed tool call so it persists in the streaming UI
          const toolName = activeToolRef.current;
          const meta = activeToolMetaRef.current;
          if (toolName && meta) {
            // Try to parse the final input
            setStreamingToolInput((prevInput) => {
              let parsedInput: unknown = {};
              if (prevInput) {
                try { parsedInput = JSON.parse(prevInput); } catch { parsedInput = {}; }
              }
              setCompletedToolCalls((prev) => [
                ...prev,
                {
                  id: meta.id,
                  name: toolName,
                  input: parsedInput,
                  agentName: meta.agentName ?? undefined,
                },
              ]);
              return "";
            });
          } else {
            setStreamingToolInput("");
          }
          setActiveTool(null);
          activeToolRef.current = null;
          activeToolMetaRef.current = null;
          break;
        }
        case "subagent_start": {
          const data = event.data as { name?: string };
          const subName = data.name ?? "subagent";
          setActiveSubagent(subName);
          activeSubagentRef.current = subName;
          setIsProcessing(true);
          setOptimisticMsg(null);
          break;
        }
        case "subagent_end":
          setActiveSubagent(null);
          activeSubagentRef.current = null;
          break;
        case "permission_request": {
          const data = event.data as PermissionRequestData;
          if (data.request_id) {
            setPendingPermissions((prev) => {
              const next = new Map(prev);
              next.set(data.request_id, data);
              return next;
            });
          }
          break;
        }
        case "permission_response": {
          const data = event.data as { request_id?: string; approved?: boolean };
          if (data.request_id) {
            setPendingPermissions((prev) => {
              const next = new Map(prev);
              const entry = next.get(data.request_id!);
              if (entry) {
                next.set(data.request_id!, {
                  ...entry,
                  status: data.approved ? "approved" : "denied",
                });
              }
              return next;
            });
            if (!data.approved) {
              // Agent was interrupted by denial
              setIsProcessing(false);
              setActiveTool(null);
              refetchSession();
            }
          }
          break;
        }
        case "user_input_request": {
          const data = event.data as UserInputRequestData;
          setPendingUserQuestions(data);
          setIsProcessing(false);
          break;
        }
        case "tool_progress": {
          const data = event.data as { progress?: string };
          if (data.progress) {
            setToolProgressText(data.progress);
          }
          break;
        }
        case "result_error": {
          const data = event.data as { error_type?: string; message?: string; cost_usd?: number };
          setResultError({
            error_type: data.error_type ?? "unknown",
            message: data.message ?? "Agent encountered an error",
            cost_usd: data.cost_usd,
          });
          setIsProcessing(false);
          setActiveTool(null);
          setActiveSubagent(null);
          setOptimisticMsg(null);
          refetchSession();
          break;
        }
        case "queue_updated": {
          const data = event.data as { queue?: Array<{ id: string; prompt_text: string; queued_at: string }>; queue_length?: number };
          setMessageQueue(data.queue ?? []);
          break;
        }
        case "file_changed": {
          const data = event.data as { tool_name?: string; file_path?: string; action?: string };
          if (data.file_path) {
            setFileActivities((prev) => [
              ...prev,
              {
                id: event.id,
                tool_name: data.tool_name ?? "unknown",
                file_path: data.file_path!,
                action: data.action ?? "accessed",
                timestamp: new Date(),
              },
            ]);
          }
          break;
        }
        case "session_completed":
        case "session_failed":
        case "error": {
          // Give the backend a moment to persist the assistant message
          // before refetching — prevents the "vanishing message" flash
          // where streaming text is cleared before the DB write completes.
          const clearState = () => {
            setStreamingText("");
            setThinkingText("");
            setIsProcessing(false);
            setActiveTool(null);
            setActiveSubagent(null);
            setOptimisticMsg(null);
            setPendingUserQuestions(null);
            setToolProgressText(null);
            setResultError(null);
            setLiveTodos([]);
            setCompletedToolCalls([]);
          };
          // Retry refetch with a short delay to ensure DB persistence
          const tryRefetch = (attempts: number) => {
            setTimeout(() => {
              refetchSession().then((result) => {
                // Check if we got the assistant message back
                const msgs = result.data?.messages ?? [];
                const lastMsg = msgs[msgs.length - 1];
                if (lastMsg?.role === "assistant" || attempts >= 3) {
                  clearState();
                } else {
                  // Message not persisted yet — retry
                  tryRefetch(attempts + 1);
                }
              });
            }, attempts === 0 ? 200 : 500);
          };
          tryRefetch(0);
          break;
        }
      }
    },
    [refetchSession]
  );

  const { connected: streamConnected } = useSessionStream(activeSessionId, handleStreamEvent);

  // Smart auto-scroll
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const handleScroll = () => {
      const distFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
      isNearBottomRef.current = distFromBottom < 100;
      setShowScrollButton(distFromBottom > 200);
    };
    el.addEventListener("scroll", handleScroll, { passive: true });
    return () => el.removeEventListener("scroll", handleScroll);
  }, []);

  useEffect(() => {
    if (isNearBottomRef.current) {
      scrollRef.current?.scrollTo({
        top: scrollRef.current.scrollHeight,
        behavior: "smooth",
      });
    }
  }, [session?.messages, streamingText, thinkingText, optimisticMsg]);

  const scrollToBottom = useCallback(() => {
    scrollRef.current?.scrollTo({
      top: scrollRef.current.scrollHeight,
      behavior: "smooth",
    });
  }, []);

  const handleSaveSkills = useCallback(
    (skills: SessionSkill[]) => {
      if (activeSessionId) {
        updateSessionSkills.mutate(skills);
      }
    },
    [activeSessionId, updateSessionSkills]
  );

  // Derive active skills from session metadata
  const activeSkills: SessionSkill[] = useMemo(() => {
    const raw = session?.metadata?.active_skills;
    if (!Array.isArray(raw)) return [];
    return raw as SessionSkill[];
  }, [session?.metadata]);

  // Workspace skills — derived from session metadata if the workspace stored them there.
  const workspaceSkills = useMemo(() => {
    const raw = session?.metadata?.workspace_skills;
    if (!Array.isArray(raw)) return [];
    return raw as Array<{ source: string; name?: string; enabled?: boolean }>;
  }, [session?.metadata]);

  const handleSend = useCallback(
    (text: string, images?: ImageAttachment[]) => {
      // Intercept /skills command client-side — open picker dialog.
      if (text.trim() === "/skills") {
        setShowSkillPicker(true);
        return;
      }

      // Show optimistic message immediately
      setOptimisticMsg({
        text,
        images: images?.map((img) => ({ data: img.data, media_type: img.media_type })),
        timestamp: new Date(),
      });
      setIsProcessing(true);
      setLastCost(null);
      setLastDuration(null);

      // Scroll to bottom to show the optimistic message
      setTimeout(() => {
        scrollRef.current?.scrollTo({
          top: scrollRef.current.scrollHeight,
          behavior: "smooth",
        });
      }, 50);

      const payload: SendMessagePayload = {
        text,
        images: images?.map((img) => ({
          data: img.data,
          media_type: img.media_type,
        })),
      };

      sendMessage.mutate(payload, {
        onSuccess: () => {
          // The backend now returns immediately (non-blocking). The agent
          // runs in the background and streams events via SSE. The
          // session_completed / result_error SSE events handle cleanup.
          // Just refetch to pick up the persisted user message.
          refetchSession();
        },
        onError: () => {
          // Message failed to send — clear all transient state.
          setStreamingText("");
          setThinkingText("");
          setIsProcessing(false);
          setActiveTool(null);
          setActiveSubagent(null);
          setOptimisticMsg(null);
          setToolProgressText(null);
          setResultError(null);
        },
      });
    },
    [sendMessage, refetchSession]
  );

  const clearTransientState = useCallback(() => {
    setStreamingText("");
    setThinkingText("");
    setIsProcessing(false);
    setActiveTool(null);
    activeToolRef.current = null;
    setActiveSubagent(null);
    setLastCost(null);
    setLastDuration(null);
    setTotalCost(0);
    setOptimisticMsg(null);
    setPendingUserQuestions(null);
    setToolProgressText(null);
    setResultError(null);
    setMessageQueue([]);
    setFileActivities([]);
    setLiveTodos([]);
    setStreamingToolInput("");
    setCompletedToolCalls([]);
    activeSubagentRef.current = null;
    activeToolMetaRef.current = null;
  }, []);

  const handleNewSession = useCallback(() => {
    createSession.mutate(undefined, {
      onSuccess: (newSession) => {
        setActiveSessionId(newSession.id);
        clearTransientState();
        refetchSessions();
      },
    });
  }, [createSession, refetchSessions, clearTransientState]);

  const switchSession = useCallback((sessionId: string) => {
    setActiveSessionId(sessionId);
    clearTransientState();
  }, [clearTransientState]);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!sortedSessions) return;
      if ((e.metaKey || e.ctrlKey) && e.key >= "1" && e.key <= "9") {
        const index = parseInt(e.key) - 1;
        if (index < sortedSessions.length) {
          e.preventDefault();
          switchSession(sortedSessions[index].id);
        }
      }
      if ((e.metaKey || e.ctrlKey) && e.key === "n") {
        e.preventDefault();
        handleNewSession();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [sortedSessions, switchSession, handleNewSession]);

  const hasMessages = (session?.messages?.length ?? 0) > 0 || optimisticMsg !== null;

  // No sessions at all
  if (!activeSessionId) {
    return <WelcomeScreen onNewSession={handleNewSession} />;
  }

  return (
    <div className="flex flex-col h-full">
      {/* Session bar */}
      <SessionBar
        sessions={sortedSessions}
        activeSessionId={activeSessionId}
        onSwitch={switchSession}
        onNew={handleNewSession}
        isProcessing={isProcessing}
        activeTool={activeTool}
        activeSubagent={activeSubagent}
        isThinking={!!thinkingText && !streamingText}
        lastCost={lastCost}
        lastDuration={lastDuration}
        totalCost={totalCost}
        permissionMode={permissionMode}
        onPermissionModeChange={handlePermissionModeChange}
        showActivityPanel={showActivityPanel}
        onToggleActivityPanel={() => setShowActivityPanel((p) => !p)}
        hasActivity={fileActivities.length > 0}
      />

      {/* Main content area: chat + optional activity panel */}
      <div className="flex-1 min-h-0 flex">
      {/* Chat column */}
      <div className={`flex-1 min-w-0 flex flex-col ${showActivityPanel && fileActivities.length > 0 ? "" : ""}`}>

      {/* Messages area */}
      <div className="flex-1 min-h-0 relative">
        <div
          ref={scrollRef}
          className="absolute inset-0 overflow-y-auto px-3 py-4 space-y-4"
        >
          {/* Empty session prompt */}
          {!hasMessages && !isProcessing && !streamingText && (
            <SessionEmptyState onSend={handleSend} />
          )}

          {/* Persisted messages */}
          {session?.messages.map((msg: Message) => (
            <MessageBubble
              key={msg.id}
              message={msg}
              onRetry={
                msg.role === "user"
                  ? () => {
                      const textContent = msg.content.find(
                        (c) => c.type === "text"
                      );
                      if (textContent && textContent.type === "text") {
                        handleSend(textContent.text);
                      }
                    }
                  : undefined
              }
            />
          ))}

          {/* Optimistic user message (shown immediately on send) */}
          {optimisticMsg && (
            <div className="flex gap-3 flex-row-reverse animate-fade-in">
              <div className="w-7 h-7 rounded-lg bg-ciab-steel-blue/10 ring-1 ring-ciab-steel-blue/20 flex items-center justify-center flex-shrink-0 mt-1">
                <User className="w-3.5 h-3.5 text-ciab-steel-blue" />
              </div>
              <div className="flex-1 min-w-0 flex flex-col items-end space-y-2">
                <span className="text-[10px] font-mono font-medium tracking-wide text-ciab-steel-blue/60">
                  YOU
                </span>
                {/* Images */}
                {optimisticMsg.images?.map((img, i) => (
                  <div key={i} className="rounded-xl border border-ciab-border overflow-hidden max-w-[80%] bg-ciab-bg-card">
                    <img
                      src={`data:${img.media_type};base64,${img.data}`}
                      alt="Attached"
                      className="max-w-full max-h-[200px] object-contain"
                    />
                  </div>
                ))}
                {/* Text */}
                {optimisticMsg.text && (
                  <div className="rounded-xl px-4 py-3 max-w-[90%] bg-ciab-copper/8 border border-ciab-copper/15">
                    <p className="text-sm leading-relaxed">{optimisticMsg.text}</p>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Pending permission confirmations */}
          {Array.from(pendingPermissions.entries()).map(([reqId, perm]) => (
            <PermissionConfirmation
              key={reqId}
              requestId={reqId}
              toolName={perm.tool_name}
              toolInput={perm.tool_input}
              riskLevel={perm.risk_level}
              status={perm.status}
              onApprove={handlePermissionApprove}
              onDeny={handlePermissionDeny}
              onAllowTool={handleAllowTool}
              onSwitchMode={handleSwitchMode}
            />
          ))}

          {/* User input request (AskUserQuestion) */}
          {pendingUserQuestions && (
            <UserInputRequest
              data={pendingUserQuestions}
              onSubmit={(answer) => {
                const requestId = pendingUserQuestions.request_id ?? pendingUserQuestions.tool_use_id;
                setPendingUserQuestions(null);
                if (requestId) {
                  respondToUserInput.mutate({ requestId, answer });
                } else {
                  // Fallback for legacy flow without request_id
                  handleSend(answer);
                }
              }}
            />
          )}

          {/* Result error banner */}
          {resultError && (
            <div className="flex gap-2 sm:gap-3 animate-fade-in">
              <div className="w-6 h-6 sm:w-7 sm:h-7 rounded-lg bg-red-500/10 ring-1 ring-red-500/20 flex items-center justify-center flex-shrink-0 mt-1">
                <AlertTriangle className="w-3 h-3 sm:w-3.5 sm:h-3.5 text-red-400" />
              </div>
              <div className="flex-1 min-w-0">
                <span className="text-[10px] font-mono font-medium tracking-wide text-red-400/60">
                  ERROR
                </span>
                <div className="mt-1 rounded-xl px-3 py-2.5 border border-red-500/20 bg-red-500/[0.05]">
                  <p className="text-sm font-medium text-red-400 mb-1">
                    {resultError.error_type === "error_max_budget_usd"
                      ? "Budget exceeded"
                      : resultError.error_type === "error_max_turns"
                        ? "Max turns reached"
                        : "Agent error"}
                  </p>
                  <p className="text-xs text-ciab-text-muted">
                    {resultError.message}
                  </p>
                  {resultError.cost_usd != null && (
                    <p className="text-[10px] font-mono text-ciab-text-muted/60 mt-1">
                      Cost: ${resultError.cost_usd.toFixed(4)}
                    </p>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Thinking block (collapsible, compact) */}
          {thinkingText && !streamingText && (
            <ThinkingBlock text={thinkingText} />
          )}

          {/* Subagent activity */}
          {activeSubagent && (
            <SubagentIndicator name={activeSubagent} />
          )}

          {/* Live todo list */}
          {liveTodos.length > 0 && (
            <TodoListBlock todos={liveTodos} isLive={isProcessing} />
          )}

          {/* Completed tool calls from this streaming turn */}
          {completedToolCalls.length > 0 && (
            <div className="flex gap-2 sm:gap-3 animate-fade-in">
              <div className="w-6 h-6 sm:w-7 sm:h-7 rounded-lg bg-ciab-copper/10 ring-1 ring-ciab-copper/20 flex items-center justify-center flex-shrink-0 mt-1">
                <AgentProviderIcon provider="claude-code" size={14} />
              </div>
              <div className="flex-1 min-w-0 space-y-1.5">
                <span className="text-[10px] font-mono font-medium tracking-wide text-ciab-copper/60">
                  AGENT
                </span>
                {completedToolCalls.map((tc) => (
                  <div key={tc.id}>
                    {tc.agentName && (
                      <span className="text-[9px] font-mono text-violet-400/70 mb-0.5 block">
                        ↳ {tc.agentName}
                      </span>
                    )}
                    <ToolUseBlock name={tc.name} input={tc.input} toolId={tc.id} />
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Streaming text with markdown */}
          {streamingText && <StreamingMessage text={streamingText} thinkingText={thinkingText} />}

          {/* Live tool call — show the actual ToolUseBlock with streaming input */}
          {isProcessing && activeTool && (
            <div className="flex gap-2 sm:gap-3 animate-fade-in">
              <div className="w-6 h-6 sm:w-7 sm:h-7 rounded-lg bg-ciab-copper/10 ring-1 ring-ciab-copper/20 flex items-center justify-center flex-shrink-0 mt-1">
                <AgentProviderIcon provider="claude-code" size={14} />
              </div>
              <div className="flex-1 min-w-0">
                {activeSubagent && (
                  <span className="text-[9px] font-mono text-violet-400/70 mb-0.5 block">
                    ↳ {activeSubagent}
                  </span>
                )}
                <span className="text-[10px] font-mono font-medium tracking-wide text-ciab-copper/60">
                  AGENT
                </span>
                <div className="mt-1">
                  <ToolUseBlock
                    name={activeTool}
                    input={(() => {
                      if (!streamingToolInput) return {};
                      try { return JSON.parse(streamingToolInput); } catch { return {}; }
                    })()}
                    isExecuting
                  />
                </div>
              </div>
            </div>
          )}

          {/* Processing indicator (no active tool — between tools or thinking) */}
          {isProcessing && !activeTool && !streamingText && !thinkingText && (
            <ThinkingIndicator activeTool={null} toolProgress={toolProgressText} />
          )}
        </div>

        {/* Scroll to bottom */}
        {showScrollButton && (
          <button
            onClick={scrollToBottom}
            className="absolute bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-1.5 px-3 py-1.5
              rounded-full bg-ciab-bg-elevated/95 border border-ciab-border shadow-xl backdrop-blur-sm
              hover:bg-ciab-bg-hover transition-all hover:scale-105 animate-slide-up z-10
              text-[11px] font-mono text-ciab-text-muted"
          >
            <ArrowDown className="w-3 h-3" />
            New messages
          </button>
        )}
      </div>

      {/* Queue indicator */}
      {messageQueue.length > 0 && (
        <div className="mx-4 mb-1 flex items-center gap-2 text-xs text-[var(--ciab-text-muted)]">
          <div className="h-1.5 w-1.5 rounded-full bg-amber-500 animate-pulse" />
          <span>{messageQueue.length} message{messageQueue.length > 1 ? "s" : ""} queued</span>
          {messageQueue.map((m) => (
            <span
              key={m.id}
              className="inline-flex items-center gap-1 rounded bg-[var(--ciab-bg-elevated)] px-1.5 py-0.5"
            >
              <span className="max-w-[120px] truncate">{m.prompt_text}</span>
              <button
                onClick={() => {
                  if (activeSessionId) {
                    sessions.cancelQueuedMessage(activeSessionId, m.id);
                  }
                }}
                className="text-[var(--ciab-text-muted)] hover:text-red-400 transition-colors"
                title="Cancel"
              >
                ×
              </button>
            </span>
          ))}
        </div>
      )}
      {/* Connection status + Input */}
      {!streamConnected && activeSessionId && (
        <div className="mx-3 mb-1 flex items-center gap-1.5 text-[10px] font-mono text-amber-400/70">
          <div className="w-1.5 h-1.5 rounded-full bg-amber-400/70 animate-pulse" />
          Connecting to stream...
        </div>
      )}
      <ChatInput
        onSend={handleSend}
        disabled={false}
        isProcessing={isProcessing}
        sessionId={activeSessionId}
        agentProvider={agentProvider}
      />

      {/* Skill picker dialog */}
      <SkillPicker
        isOpen={showSkillPicker}
        onClose={() => setShowSkillPicker(false)}
        activeSkills={activeSkills}
        workspaceSkills={workspaceSkills}
        onSave={handleSaveSkills}
      />
      </div>{/* end chat column */}

      {/* Activity panel — right side */}
      {showActivityPanel && (
        <div className="w-64 xl:w-72 flex-shrink-0 border-l border-ciab-border bg-ciab-bg-secondary/30 hidden md:block">
          <ActivityPanel
            activities={fileActivities}
            isProcessing={isProcessing}
            activeTool={activeTool}
          />
        </div>
      )}
      </div>{/* end main content flex */}
    </div>
  );
}

/* ─── Session Bar ─── */

function SessionBar({
  sessions,
  activeSessionId,
  onSwitch,
  onNew,
  isProcessing,
  activeTool,
  activeSubagent,
  isThinking,
  lastCost,
  lastDuration,
  totalCost,
  permissionMode,
  onPermissionModeChange,
  showActivityPanel,
  onToggleActivityPanel,
  hasActivity,
}: {
  sessions: Session[];
  activeSessionId: string;
  onSwitch: (id: string) => void;
  onNew: () => void;
  isProcessing: boolean;
  activeTool: string | null;
  activeSubagent: string | null;
  isThinking: boolean;
  lastCost: number | null;
  lastDuration: number | null;
  totalCost: number;
  permissionMode: PermissionMode;
  onPermissionModeChange: (mode: PermissionMode) => void;
  showActivityPanel: boolean;
  onToggleActivityPanel: () => void;
  hasActivity: boolean;
}) {
  void totalCost;

  // Current activity label for compact mobile display
  const activityLabel = activeTool
    ? activeTool
    : activeSubagent
    ? `↳ ${activeSubagent}`
    : isThinking
    ? "Thinking"
    : null;

  return (
    <div className="flex items-center gap-1 pb-2 border-b border-ciab-border flex-shrink-0 overflow-x-auto scrollbar-none">
      {/* Permission mode selector */}
      <PermissionModeSelector
        mode={permissionMode}
        onChange={onPermissionModeChange}
      />

      {/* Session tabs */}
      <div className="flex items-center gap-0.5 overflow-x-auto flex-1 min-w-0 scrollbar-none">
        {sessions.map((s, i) => {
          const isActive = s.id === activeSessionId;
          const isCompleted = s.state === "completed";
          const isFailed = s.state === "failed";
          return (
            <button
              key={s.id}
              onClick={() => onSwitch(s.id)}
              className={`group relative flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg transition-all whitespace-nowrap flex-shrink-0 ${
                isActive
                  ? "bg-ciab-bg-card text-ciab-text-primary border border-ciab-border shadow-sm"
                  : "text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover border border-transparent"
              }`}
              title={`Session ${i + 1} · ${s.id.slice(0, 8)}\n${formatRelativeTime(s.created_at)}\n⌘${i + 1}`}
            >
              <span
                className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${
                  isActive && isProcessing
                    ? "bg-ciab-copper animate-pulse"
                    : isActive
                      ? "bg-state-running"
                      : isCompleted
                        ? "bg-state-running/40"
                        : isFailed
                          ? "bg-state-failed/60"
                          : "bg-ciab-text-muted/30"
                }`}
              />
              <span className="text-[11px] font-mono font-medium">
                {i + 1}
              </span>
            </button>
          );
        })}

        <button
          onClick={onNew}
          className="flex items-center gap-1 px-2 py-1.5 rounded-lg
            text-ciab-text-muted hover:text-ciab-copper hover:bg-ciab-copper/5
            transition-all flex-shrink-0 border border-transparent hover:border-ciab-copper/20"
          title="New session (⌘N)"
        >
          <Plus className="w-3 h-3" />
        </button>
      </div>

      {/* Right side metrics — compact on mobile */}
      <div className="flex items-center gap-1.5 sm:gap-2 flex-shrink-0 pl-2 border-l border-ciab-border/50">
        {/* Activity indicator — single compact pill */}
        {activityLabel && (
          <div className="flex items-center gap-1 px-1.5 sm:px-2 py-1 rounded-md bg-ciab-copper/5 border border-ciab-copper/15 animate-fade-in max-w-[120px] sm:max-w-none">
            <span className="w-1.5 h-1.5 rounded-full bg-ciab-copper animate-pulse flex-shrink-0" />
            <span className="text-[10px] font-mono text-ciab-copper font-medium truncate">
              {activityLabel}
            </span>
          </div>
        )}

        {isProcessing && !activityLabel && (
          <div className="flex items-center gap-0.5 px-1.5 py-1">
            <div className="w-1 h-1 rounded-full bg-ciab-copper/80 animate-bounce" style={{ animationDelay: "0ms", animationDuration: "0.8s" }} />
            <div className="w-1 h-1 rounded-full bg-ciab-copper/80 animate-bounce" style={{ animationDelay: "100ms", animationDuration: "0.8s" }} />
            <div className="w-1 h-1 rounded-full bg-ciab-copper/80 animate-bounce" style={{ animationDelay: "200ms", animationDuration: "0.8s" }} />
          </div>
        )}

        {!isProcessing && (lastDuration || lastCost) && (
          <div className="hidden sm:flex items-center gap-2">
            {lastDuration != null && (
              <span className="flex items-center gap-1 text-[10px] font-mono text-ciab-text-muted/60">
                <Clock className="w-2.5 h-2.5" />
                {(lastDuration / 1000).toFixed(1)}s
              </span>
            )}
            {lastCost != null && (
              <span className="flex items-center gap-1 text-[10px] font-mono text-ciab-text-muted/60">
                <DollarSign className="w-2.5 h-2.5" />
                ${lastCost.toFixed(4)}
              </span>
            )}
          </div>
        )}

        {/* Activity panel toggle */}
        <button
          onClick={onToggleActivityPanel}
          className={`hidden md:flex items-center gap-1 p-1.5 rounded-md transition-colors ${
            showActivityPanel && hasActivity
              ? "text-ciab-copper bg-ciab-copper/10"
              : "text-ciab-text-muted hover:text-ciab-text-secondary hover:bg-ciab-bg-hover"
          }`}
          title={showActivityPanel ? "Hide activity panel" : "Show activity panel"}
        >
          {showActivityPanel ? (
            <PanelRightClose className="w-3.5 h-3.5" />
          ) : (
            <PanelRightOpen className="w-3.5 h-3.5" />
          )}
          {hasActivity && !showActivityPanel && (
            <span className="w-1.5 h-1.5 rounded-full bg-ciab-copper animate-pulse" />
          )}
        </button>
      </div>
    </div>
  );
}

/* ─── Welcome Screen ─── */

function WelcomeScreen({ onNewSession }: { onNewSession: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center h-full text-center px-8">
      <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-ciab-copper/10 to-ciab-copper/5 border border-ciab-copper/20 flex items-center justify-center mb-6 shadow-lg shadow-ciab-copper/5">
        <AgentProviderIcon provider="claude-code" size={32} />
      </div>
      <h2 className="text-lg font-semibold text-ciab-text-primary mb-2">
        Start a conversation
      </h2>
      <p className="text-sm text-ciab-text-muted mb-8 max-w-[320px] leading-relaxed">
        Create a session to chat with the agent. It can edit files, run commands, search code, and more.
      </p>
      <button
        onClick={onNewSession}
        className="btn-primary flex items-center gap-2.5 text-sm px-6 py-3 rounded-xl shadow-lg shadow-ciab-copper/20 hover:shadow-ciab-copper/30 transition-all"
      >
        <Zap className="w-4 h-4" />
        New Session
      </button>
      <p className="text-[10px] font-mono text-ciab-text-muted/40 mt-4">
        <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated text-[9px]">{"\u2318N"}</kbd> to create
      </p>
    </div>
  );
}

/* ─── Session Empty State (quick prompts) ─── */

function SessionEmptyState({ onSend }: { onSend: (text: string) => void }) {
  const quickPrompts = [
    {
      icon: Sparkles,
      label: "Explain the codebase",
      prompt: "Give me a high-level overview of this codebase. What are the main components and how do they fit together?",
      color: "text-ciab-copper",
    },
    {
      icon: Search,
      label: "Find a bug",
      prompt: "Help me find and fix any bugs in this code. Start by looking at the most recent changes.",
      color: "text-ciab-steel-blue",
    },
    {
      icon: FileEdit,
      label: "Add a feature",
      prompt: "I want to add a new feature. Let me describe what I need...",
      color: "text-state-running",
    },
    {
      icon: Terminal,
      label: "Run tests",
      prompt: "Run the test suite and report any failures. If there are failures, help me fix them.",
      color: "text-state-paused",
    },
    {
      icon: GitBranch,
      label: "Review changes",
      prompt: "Review the recent git changes and give me feedback on code quality, potential issues, and improvements.",
      color: "text-violet-400",
    },
  ];

  return (
    <div className="flex flex-col items-center justify-center min-h-[300px] py-8">
      <div className="w-12 h-12 rounded-xl bg-ciab-bg-card border border-ciab-border flex items-center justify-center mb-4">
        <MessageSquare className="w-5 h-5 text-ciab-text-muted/40" />
      </div>
      <p className="text-sm text-ciab-text-secondary font-medium mb-1">
        What would you like to do?
      </p>
      <p className="text-xs text-ciab-text-muted mb-6">
        Type a message or try one of these:
      </p>

      <div className="flex flex-wrap justify-center gap-2 max-w-lg">
        {quickPrompts.map((qp) => (
          <button
            key={qp.label}
            onClick={() => onSend(qp.prompt)}
            className="flex items-center gap-2 px-3.5 py-2 rounded-xl
              bg-ciab-bg-card border border-ciab-border
              hover:border-ciab-border-light hover:bg-ciab-bg-elevated
              transition-all text-left group"
          >
            <qp.icon className={`w-3.5 h-3.5 ${qp.color} flex-shrink-0 opacity-60 group-hover:opacity-100 transition-opacity`} />
            <span className="text-[12px] text-ciab-text-secondary group-hover:text-ciab-text-primary transition-colors">
              {qp.label}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}

/* ─── Streaming Message (with markdown + live cursor) ─── */

function StreamingMessage({ text, thinkingText }: { text: string; thinkingText?: string }) {
  const [thinkingExpanded, setThinkingExpanded] = useState(false);

  return (
    <div className="flex gap-2 sm:gap-3 animate-fade-in">
      <div className="w-6 h-6 sm:w-7 sm:h-7 rounded-lg bg-ciab-copper/10 ring-1 ring-ciab-copper/20 flex items-center justify-center flex-shrink-0 mt-1">
        <AgentProviderIcon provider="claude-code" size={14} />
      </div>
      <div className="flex-1 min-w-0">
        <span className="text-[10px] font-mono font-medium tracking-wide text-ciab-copper/60">
          AGENT
        </span>

        {/* Collapsed thinking summary above response */}
        {thinkingText && (
          <button
            onClick={() => setThinkingExpanded(!thinkingExpanded)}
            className="flex items-center gap-1.5 mt-1 mb-1 text-[10px] font-mono text-violet-400/60 hover:text-violet-400/80 transition-colors"
          >
            {thinkingExpanded ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
            <Brain className="w-3 h-3" />
            Thought for a moment
          </button>
        )}
        {thinkingText && thinkingExpanded && (
          <div className="rounded-lg px-3 py-2 mb-2 bg-violet-500/[0.03] border border-violet-500/10 animate-fade-in">
            <p className="text-[11px] text-ciab-text-muted/70 leading-relaxed whitespace-pre-wrap font-mono max-h-[200px] overflow-y-auto scrollbar-none">
              {thinkingText}
            </p>
          </div>
        )}

        <div className="bg-ciab-bg-card rounded-xl px-3 py-3 sm:p-4 border border-ciab-border mt-1">
          <div className="text-sm prose prose-invert prose-sm max-w-none leading-relaxed
            prose-p:my-1.5 prose-code:text-ciab-copper-light prose-code:bg-ciab-bg-primary/80
            prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded-md prose-code:text-xs prose-code:font-mono
            prose-code:before:content-none prose-code:after:content-none
            prose-pre:bg-ciab-bg-primary prose-pre:border prose-pre:border-ciab-border prose-pre:rounded-md prose-pre:p-3 prose-pre:my-3
            prose-strong:text-ciab-text-primary prose-headings:text-ciab-text-primary">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>
              {text}
            </ReactMarkdown>
            <span className="inline-block w-[3px] h-[18px] bg-ciab-copper animate-pulse ml-0.5 rounded-full align-middle" />
          </div>
        </div>
      </div>
    </div>
  );
}

/* ─── Thinking Block (shown while thinking, before text arrives) ─── */

function ThinkingBlock({ text }: { text: string }) {
  const [expanded, setExpanded] = useState(false);
  const lines = text.split("\n").length;
  const preview = text.length > 120 ? text.slice(0, 120) + "..." : text;

  return (
    <div className="flex gap-2 sm:gap-3 animate-fade-in">
      <div className="w-6 h-6 sm:w-7 sm:h-7 rounded-lg bg-violet-500/10 ring-1 ring-violet-500/20 flex items-center justify-center flex-shrink-0 mt-1">
        <Brain className="w-3 h-3 sm:w-3.5 sm:h-3.5 text-violet-400" />
      </div>
      <div className="flex-1 min-w-0">
        <span className="text-[10px] font-mono font-medium tracking-wide text-violet-400/60">
          THINKING
        </span>
        <div className="mt-1 rounded-lg border border-violet-500/10 bg-violet-500/[0.03] overflow-hidden">
          <button
            onClick={() => setExpanded(!expanded)}
            className="w-full text-left px-3 py-2 flex items-center gap-2 hover:bg-violet-500/[0.03] transition-colors"
          >
            {expanded ? <ChevronDown className="w-3 h-3 text-violet-400/50 flex-shrink-0" /> : <ChevronRight className="w-3 h-3 text-violet-400/50 flex-shrink-0" />}
            <p className="text-[11px] text-ciab-text-muted/60 font-mono truncate flex-1">
              {expanded ? `${lines} lines` : preview}
            </p>
            <div className="flex items-center gap-0.5 flex-shrink-0">
              <div className="w-1 h-1 rounded-full bg-violet-400/50 animate-bounce" style={{ animationDelay: "0ms", animationDuration: "1s" }} />
              <div className="w-1 h-1 rounded-full bg-violet-400/50 animate-bounce" style={{ animationDelay: "200ms", animationDuration: "1s" }} />
              <div className="w-1 h-1 rounded-full bg-violet-400/50 animate-bounce" style={{ animationDelay: "400ms", animationDuration: "1s" }} />
            </div>
          </button>
          {expanded && (
            <div className="px-3 pb-2 animate-fade-in">
              <p className="text-[11px] text-ciab-text-muted/60 leading-relaxed whitespace-pre-wrap font-mono max-h-[300px] overflow-y-auto scrollbar-none">
                {text}
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

/* ─── Subagent Indicator ─── */

function SubagentIndicator({ name }: { name: string }) {
  return (
    <div className="flex gap-2 sm:gap-3 animate-fade-in">
      <div className="w-6 h-6 sm:w-7 sm:h-7 rounded-lg bg-ciab-steel-blue/10 ring-1 ring-ciab-steel-blue/20 flex items-center justify-center flex-shrink-0 mt-1">
        <Bot className="w-3 h-3 sm:w-3.5 sm:h-3.5 text-ciab-steel-blue" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="mt-1 inline-flex items-center gap-2 px-3 py-1.5 rounded-lg bg-ciab-steel-blue/5 border border-ciab-steel-blue/15">
          <span className="w-1.5 h-1.5 rounded-full bg-ciab-steel-blue animate-pulse" />
          <span className="text-[11px] font-mono text-ciab-steel-blue">
            Subagent: <span className="font-medium">{name}</span>
          </span>
        </div>
      </div>
    </div>
  );
}

/* ─── Thinking Indicator (generic, no text yet) ─── */

function ThinkingIndicator({ activeTool, toolProgress }: { activeTool: string | null; toolProgress?: string | null }) {
  return (
    <div className="flex gap-2 sm:gap-3 animate-fade-in">
      <div className="w-6 h-6 sm:w-7 sm:h-7 rounded-lg bg-ciab-copper/10 ring-1 ring-ciab-copper/20 flex items-center justify-center flex-shrink-0 mt-1">
        <AgentProviderIcon provider="claude-code" size={14} />
      </div>
      <div className="flex-1 min-w-0">
        <span className="text-[10px] font-mono font-medium tracking-wide text-ciab-copper/60">
          AGENT
        </span>
        <div className="bg-ciab-bg-card rounded-xl px-3 py-2.5 border border-ciab-border mt-1 inline-flex items-center gap-2">
          <div className="flex items-center gap-0.5">
            <div className="w-1.5 h-1.5 rounded-full bg-ciab-copper/70 animate-bounce" style={{ animationDelay: "0ms", animationDuration: "0.9s" }} />
            <div className="w-1.5 h-1.5 rounded-full bg-ciab-copper/70 animate-bounce" style={{ animationDelay: "150ms", animationDuration: "0.9s" }} />
            <div className="w-1.5 h-1.5 rounded-full bg-ciab-copper/70 animate-bounce" style={{ animationDelay: "300ms", animationDuration: "0.9s" }} />
          </div>
          <span className="text-[11px] text-ciab-text-muted font-mono">
            {activeTool ? (
              <>
                <span className="text-ciab-copper">{activeTool}</span>
                {toolProgress && (
                  <span className="text-ciab-text-muted/60 ml-1.5">{toolProgress}</span>
                )}
              </>
            ) : (
              "..."
            )}
          </span>
        </div>
      </div>
    </div>
  );
}
