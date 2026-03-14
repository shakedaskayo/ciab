import { useState } from "react";
import { HelpCircle, Send, Check } from "lucide-react";
import type { UserInputRequestData, UserInputQuestion } from "@/lib/api/types";

interface Props {
  data: UserInputRequestData;
  onSubmit: (answer: string) => void;
}

export default function UserInputRequest({ data, onSubmit }: Props) {
  return (
    <div className="flex gap-2 sm:gap-3 animate-fade-in">
      <div className="w-6 h-6 sm:w-7 sm:h-7 rounded-lg bg-ciab-steel-blue/10 ring-1 ring-ciab-steel-blue/20 flex items-center justify-center flex-shrink-0 mt-1">
        <HelpCircle className="w-3 h-3 sm:w-3.5 sm:h-3.5 text-ciab-steel-blue" />
      </div>
      <div className="flex-1 min-w-0 space-y-2">
        <span className="text-[10px] font-mono font-medium tracking-wide text-ciab-steel-blue/60">
          AGENT QUESTION
        </span>
        {data.questions?.map((q, i) => (
          <QuestionCard key={i} question={q} onSubmit={onSubmit} />
        ))}
      </div>
    </div>
  );
}

function QuestionCard({
  question,
  onSubmit,
}: {
  question: UserInputQuestion;
  onSubmit: (answer: string) => void;
}) {
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [freeText, setFreeText] = useState("");
  const hasOptions = question.options && question.options.length > 0;

  const handleOptionToggle = (label: string) => {
    if (question.multiSelect) {
      setSelected((prev) => {
        const next = new Set(prev);
        if (next.has(label)) next.delete(label);
        else next.add(label);
        return next;
      });
    } else {
      // Single select — submit immediately
      onSubmit(label);
    }
  };

  const handleSubmit = () => {
    if (hasOptions && selected.size > 0) {
      onSubmit(Array.from(selected).join(", "));
    } else if (freeText.trim()) {
      onSubmit(freeText.trim());
    }
  };

  return (
    <div className="rounded-xl border border-ciab-steel-blue/20 bg-ciab-bg-card overflow-hidden">
      {/* Header */}
      {question.header && (
        <div className="px-3.5 py-2 border-b border-ciab-border bg-ciab-bg-secondary/30">
          <p className="text-[11px] font-mono text-ciab-text-muted">{question.header}</p>
        </div>
      )}

      {/* Question */}
      <div className="px-3.5 py-2.5">
        <p className="text-sm text-ciab-text-primary">{question.question}</p>
      </div>

      {/* Options */}
      {hasOptions && (
        <div className="px-3.5 pb-2.5 space-y-1">
          {question.options!.map((opt) => {
            const isSelected = selected.has(opt.label);
            return (
              <button
                key={opt.label}
                onClick={() => handleOptionToggle(opt.label)}
                className={`w-full text-left flex items-center gap-2 px-3 py-2 rounded-lg text-xs font-mono transition-all ${
                  isSelected
                    ? "bg-ciab-steel-blue/10 border border-ciab-steel-blue/30 text-ciab-steel-blue"
                    : "bg-ciab-bg-secondary/30 border border-ciab-border hover:border-ciab-steel-blue/20 text-ciab-text-secondary hover:text-ciab-text-primary"
                }`}
              >
                {question.multiSelect && (
                  <span
                    className={`w-3.5 h-3.5 rounded border flex items-center justify-center flex-shrink-0 ${
                      isSelected
                        ? "border-ciab-steel-blue bg-ciab-steel-blue/20"
                        : "border-ciab-border"
                    }`}
                  >
                    {isSelected && <Check className="w-2.5 h-2.5" />}
                  </span>
                )}
                <span className="flex-1">{opt.label}</span>
                {opt.description && (
                  <span className="text-[10px] text-ciab-text-muted">{opt.description}</span>
                )}
              </button>
            );
          })}
        </div>
      )}

      {/* Free text input */}
      {(!hasOptions || question.multiSelect) && (
        <div className="px-3.5 pb-2.5">
          <div className="flex items-center gap-2">
            <input
              type="text"
              value={freeText}
              onChange={(e) => setFreeText(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  handleSubmit();
                }
              }}
              placeholder={hasOptions ? "Or type a custom answer..." : "Type your answer..."}
              className="flex-1 bg-ciab-bg-primary border border-ciab-border rounded-lg px-3 py-2
                text-xs font-mono text-ciab-text-primary placeholder:text-ciab-text-muted/40
                focus:outline-none focus:border-ciab-steel-blue/50 transition-colors"
            />
            <button
              onClick={handleSubmit}
              disabled={!freeText.trim() && selected.size === 0}
              className="flex items-center gap-1 px-3 py-2 rounded-lg text-[11px] font-mono font-medium
                bg-ciab-steel-blue/10 text-ciab-steel-blue border border-ciab-steel-blue/20
                hover:bg-ciab-steel-blue/20 transition-all
                disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <Send className="w-3 h-3" />
              Send
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
