import { useMemo } from "react";
import {
  CheckCircle2,
  Circle,
  Loader2,
  ListTodo,
} from "lucide-react";

interface TodoItem {
  id: string;
  content: string;
  status: "pending" | "in_progress" | "completed";
  priority?: "high" | "medium" | "low";
}

interface Props {
  todos: TodoItem[];
  /** True while the agent is still running (may update todos) */
  isLive?: boolean;
}

const STATUS_ICON: Record<string, typeof Circle> = {
  pending: Circle,
  in_progress: Loader2,
  completed: CheckCircle2,
};

const STATUS_STYLE: Record<string, string> = {
  pending: "text-ciab-text-muted/50",
  in_progress: "text-ciab-copper animate-spin",
  completed: "text-emerald-400",
};

const PRIORITY_DOT: Record<string, string> = {
  high: "bg-red-400",
  medium: "bg-amber-400",
  low: "bg-ciab-text-muted/30",
};

export default function TodoListBlock({ todos, isLive }: Props) {
  const { completed, total, pct } = useMemo(() => {
    const c = todos.filter((t) => t.status === "completed").length;
    const t = todos.length;
    return { completed: c, total: t, pct: t > 0 ? Math.round((c / t) * 100) : 0 };
  }, [todos]);

  return (
    <div className="rounded-xl border border-ciab-border bg-ciab-bg-card overflow-hidden max-w-[90%] animate-fade-in">
      {/* Header */}
      <div className="flex items-center gap-2 px-3.5 py-2.5 border-b border-ciab-border">
        <ListTodo className="w-4 h-4 text-ciab-copper flex-shrink-0" />
        <span className="text-xs font-mono font-semibold text-ciab-text-primary">
          Tasks
        </span>
        <span className="text-[10px] font-mono text-ciab-text-muted ml-auto">
          {completed}/{total}
        </span>
        {isLive && (
          <span className="w-1.5 h-1.5 rounded-full bg-ciab-copper animate-pulse flex-shrink-0" />
        )}
      </div>

      {/* Progress bar */}
      <div className="h-1 bg-ciab-bg-primary">
        <div
          className="h-full bg-emerald-500/60 transition-all duration-500 ease-out"
          style={{ width: `${pct}%` }}
        />
      </div>

      {/* Todo items */}
      <div className="divide-y divide-ciab-border/50">
        {todos.map((todo) => {
          const Icon = STATUS_ICON[todo.status] ?? Circle;
          const iconStyle = STATUS_STYLE[todo.status] ?? STATUS_STYLE.pending;
          const isActive = todo.status === "in_progress";
          const isDone = todo.status === "completed";

          return (
            <div
              key={todo.id}
              className={`flex items-start gap-2.5 px-3.5 py-2 transition-colors ${
                isActive
                  ? "bg-ciab-copper/[0.04]"
                  : isDone
                    ? "bg-emerald-500/[0.02]"
                    : ""
              }`}
            >
              <Icon className={`w-3.5 h-3.5 mt-0.5 flex-shrink-0 ${iconStyle}`} />

              <span
                className={`text-[12px] leading-relaxed flex-1 ${
                  isDone
                    ? "text-ciab-text-muted/50 line-through"
                    : isActive
                      ? "text-ciab-text-primary font-medium"
                      : "text-ciab-text-secondary"
                }`}
              >
                {todo.content}
              </span>

              {todo.priority && (
                <span
                  className={`w-1.5 h-1.5 rounded-full mt-1.5 flex-shrink-0 ${
                    PRIORITY_DOT[todo.priority] ?? PRIORITY_DOT.low
                  }`}
                  title={`${todo.priority} priority`}
                />
              )}
            </div>
          );
        })}
      </div>

      {/* Completion footer */}
      {completed === total && total > 0 && (
        <div className="flex items-center gap-2 px-3.5 py-2 border-t border-ciab-border bg-emerald-500/[0.04]">
          <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
          <span className="text-[11px] font-mono text-emerald-400/80">
            All tasks completed
          </span>
        </div>
      )}
    </div>
  );
}

/**
 * Parse TodoWrite input from a tool_use event.
 * Returns null if the input doesn't match the TodoWrite format.
 */
export function parseTodoInput(input: unknown): TodoItem[] | null {
  if (!input || typeof input !== "object") return null;
  const obj = input as Record<string, unknown>;
  const todos = obj.todos;
  if (!Array.isArray(todos)) return null;
  return todos.map((t: unknown) => {
    const item = t as Record<string, unknown>;
    return {
      id: String(item.id ?? ""),
      content: String(item.content ?? ""),
      status: (item.status as TodoItem["status"]) ?? "pending",
      priority: item.priority as TodoItem["priority"],
    };
  });
}
