import { useRef, useEffect, useMemo } from "react";
import {
  MessageSquare,
  Bot,
  Wrench,
  Compass,
  HelpCircle,
  Zap,
  ArrowRight,
  Command,
} from "lucide-react";
import type { SlashCommand, SlashCommandCategory } from "@/lib/api/types";

interface Props {
  commands: SlashCommand[];
  filter: string;
  selectedIndex: number;
  onSelect: (command: SlashCommand) => void;
  onClose: () => void;
}

const CATEGORY_META: Record<
  SlashCommandCategory,
  { label: string; icon: typeof MessageSquare }
> = {
  session: { label: "Session", icon: MessageSquare },
  agent: { label: "Agent", icon: Bot },
  tools: { label: "Tools", icon: Wrench },
  navigation: { label: "Navigation", icon: Compass },
  help: { label: "Help", icon: HelpCircle },
};

const CATEGORY_ORDER: SlashCommandCategory[] = [
  "session",
  "agent",
  "tools",
  "navigation",
  "help",
];

export default function SlashCommandPopup({
  commands,
  filter,
  selectedIndex,
  onSelect,
}: Props) {
  const listRef = useRef<HTMLDivElement>(null);

  const filtered = useMemo(() => {
    if (!filter) return commands;
    const lower = filter.toLowerCase();
    return commands.filter((cmd) => cmd.name.toLowerCase().startsWith(lower));
  }, [commands, filter]);

  // Group by category
  const grouped = useMemo(() => {
    const groups: {
      category: SlashCommandCategory;
      commands: SlashCommand[];
    }[] = [];
    for (const cat of CATEGORY_ORDER) {
      const cmds = filtered.filter((c) => c.category === cat);
      if (cmds.length > 0) {
        groups.push({ category: cat, commands: cmds });
      }
    }
    return groups;
  }, [filtered]);

  // Flat list for index tracking
  const flatList = useMemo(
    () => grouped.flatMap((g) => g.commands),
    [grouped]
  );

  // Scroll selected item into view
  useEffect(() => {
    const el = listRef.current?.querySelector(
      `[data-index="${selectedIndex}"]`
    );
    el?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  if (flatList.length === 0) return null;

  let runningIndex = 0;

  return (
    <div
      className="absolute bottom-full left-0 right-0 mb-2 px-3 z-50 animate-slide-up"
      onMouseDown={(e) => e.preventDefault()}
    >
      <div className="bg-ciab-bg-card border border-ciab-border rounded-xl shadow-2xl shadow-black/30 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-3 py-2 border-b border-ciab-border/50">
          <div className="flex items-center gap-2">
            <div className="flex items-center justify-center w-5 h-5 rounded-md bg-ciab-copper/10">
              <Command className="w-3 h-3 text-ciab-copper" />
            </div>
            <span className="text-[11px] font-medium text-ciab-text-secondary">
              {filter ? (
                <>
                  Commands matching{" "}
                  <span className="font-mono text-ciab-copper">/{filter}</span>
                </>
              ) : (
                "Slash Commands"
              )}
            </span>
          </div>
          <div className="flex items-center gap-1.5 text-[9px] text-ciab-text-muted/50 font-mono">
            <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated border border-ciab-border/50 text-ciab-text-muted">
              ↑↓
            </kbd>
            <span>navigate</span>
            <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated border border-ciab-border/50 text-ciab-text-muted ml-1">
              ↵
            </kbd>
            <span>select</span>
            <kbd className="px-1 py-0.5 rounded bg-ciab-bg-elevated border border-ciab-border/50 text-ciab-text-muted ml-1">
              esc
            </kbd>
            <span>dismiss</span>
          </div>
        </div>

        {/* Command list */}
        <div
          ref={listRef}
          className="max-h-[320px] overflow-y-auto overscroll-contain py-1"
        >
          {grouped.map(({ category, commands: cmds }) => {
            const meta = CATEGORY_META[category];
            const Icon = meta.icon;

            const header = (
              <div
                key={`cat-${category}`}
                className="flex items-center gap-1.5 px-3 pt-2.5 pb-1 select-none"
              >
                <Icon className="w-3 h-3 text-ciab-text-muted/40" />
                <span className="text-[10px] font-semibold tracking-widest text-ciab-text-muted/40 uppercase">
                  {meta.label}
                </span>
              </div>
            );

            const items = cmds.map((cmd) => {
              const idx = runningIndex++;
              const isSelected = idx === selectedIndex;
              const hasArgs = cmd.args.length > 0;

              return (
                <button
                  key={cmd.name}
                  data-index={idx}
                  onClick={() => onSelect(cmd)}
                  className={`group w-full flex items-center gap-2.5 px-3 py-[7px] text-left transition-all duration-100
                    ${
                      isSelected
                        ? "bg-ciab-copper/8 border-l-2 border-l-ciab-copper"
                        : "border-l-2 border-l-transparent hover:bg-ciab-bg-hover/60"
                    }`}
                >
                  {/* Command name */}
                  <span
                    className={`font-mono text-[13px] font-medium shrink-0 ${
                      isSelected ? "text-ciab-copper" : "text-ciab-text-primary"
                    }`}
                  >
                    /{cmd.name}
                  </span>

                  {/* Description */}
                  <span className="text-[11px] text-ciab-text-muted truncate flex-1">
                    {cmd.description}
                  </span>

                  {/* Right side indicators */}
                  <div className="flex items-center gap-1.5 shrink-0">
                    {hasArgs && (
                      <span className="text-[9px] font-mono px-1.5 py-0.5 rounded-md bg-ciab-bg-elevated text-ciab-text-muted/60 border border-ciab-border/30">
                        {cmd.args.map((a) => a.name).join(", ")}
                      </span>
                    )}
                    {cmd.provider_native && (
                      <Zap className="w-3 h-3 text-ciab-text-muted/30" />
                    )}
                    {isSelected && (
                      <ArrowRight className="w-3 h-3 text-ciab-copper/60" />
                    )}
                  </div>
                </button>
              );
            });

            return [header, ...items];
          })}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-3 py-1.5 border-t border-ciab-border/30 bg-ciab-bg-secondary/50">
          <span className="text-[9px] text-ciab-text-muted/40 font-mono">
            {flatList.length} command{flatList.length !== 1 ? "s" : ""}
            {filter && ` matching "/${filter}"`}
          </span>
          <div className="flex items-center gap-2 text-[9px] text-ciab-text-muted/40">
            <span className="flex items-center gap-1">
              <Zap className="w-2.5 h-2.5" />
              agent-native
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
