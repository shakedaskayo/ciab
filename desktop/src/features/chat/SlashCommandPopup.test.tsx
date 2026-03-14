import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeAll } from "vitest";
import SlashCommandPopup from "./SlashCommandPopup";
import type { SlashCommand } from "@/lib/api/types";

// jsdom doesn't implement scrollIntoView
beforeAll(() => {
  Element.prototype.scrollIntoView = vi.fn();
});

const mockCommands: SlashCommand[] = [
  {
    name: "clear",
    description: "Clear conversation history",
    category: "session",
    args: [],
    provider_native: false,
  },
  {
    name: "compact",
    description: "Compact conversation context",
    category: "session",
    args: [],
    provider_native: true,
  },
  {
    name: "model",
    description: "Switch model",
    category: "agent",
    args: [{ name: "model", description: "Model name", required: false }],
    provider_native: true,
  },
  {
    name: "help",
    description: "Show available commands",
    category: "help",
    args: [],
    provider_native: false,
  },
  {
    name: "permissions",
    description: "View/set permission mode",
    category: "tools",
    args: [],
    provider_native: true,
  },
  {
    name: "vim",
    description: "Toggle vim mode",
    category: "navigation",
    args: [],
    provider_native: true,
  },
];

describe("SlashCommandPopup", () => {
  const defaultProps = {
    commands: mockCommands,
    filter: "",
    selectedIndex: 0,
    onSelect: vi.fn(),
    onClose: vi.fn(),
  };

  it("renders all commands when filter is empty", () => {
    render(<SlashCommandPopup {...defaultProps} />);

    expect(screen.getByText("/clear")).toBeInTheDocument();
    expect(screen.getByText("/compact")).toBeInTheDocument();
    expect(screen.getByText("/model")).toBeInTheDocument();
    expect(screen.getByText("/help")).toBeInTheDocument();
    expect(screen.getByText("/permissions")).toBeInTheDocument();
    expect(screen.getByText("/vim")).toBeInTheDocument();
  });

  it("renders category headers", () => {
    render(<SlashCommandPopup {...defaultProps} />);

    expect(screen.getByText("Session")).toBeInTheDocument();
    expect(screen.getByText("Agent")).toBeInTheDocument();
    expect(screen.getByText("Tools")).toBeInTheDocument();
    expect(screen.getByText("Navigation")).toBeInTheDocument();
    expect(screen.getByText("Help")).toBeInTheDocument();
  });

  it("renders command descriptions", () => {
    render(<SlashCommandPopup {...defaultProps} />);

    expect(screen.getByText("Clear conversation history")).toBeInTheDocument();
    expect(screen.getByText("Switch model")).toBeInTheDocument();
  });

  it("filters commands by name prefix", () => {
    render(<SlashCommandPopup {...defaultProps} filter="cl" />);

    expect(screen.getByText("/clear")).toBeInTheDocument();
    expect(screen.queryByText("/compact")).not.toBeInTheDocument();
    expect(screen.queryByText("/model")).not.toBeInTheDocument();
  });

  it("filters commands case-insensitively", () => {
    render(<SlashCommandPopup {...defaultProps} filter="CL" />);

    expect(screen.getByText("/clear")).toBeInTheDocument();
  });

  it("shows multiple matches for shared prefix", () => {
    render(<SlashCommandPopup {...defaultProps} filter="c" />);

    expect(screen.getByText("/clear")).toBeInTheDocument();
    expect(screen.getByText("/compact")).toBeInTheDocument();
    expect(screen.queryByText("/model")).not.toBeInTheDocument();
  });

  it("returns null when no commands match filter", () => {
    const { container } = render(
      <SlashCommandPopup {...defaultProps} filter="zzz" />
    );

    expect(container.firstChild).toBeNull();
  });

  it("calls onSelect when a command is clicked", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();

    render(<SlashCommandPopup {...defaultProps} onSelect={onSelect} />);

    await user.click(screen.getByText("/clear"));
    expect(onSelect).toHaveBeenCalledTimes(1);
    expect(onSelect).toHaveBeenCalledWith(
      expect.objectContaining({ name: "clear" })
    );
  });

  it("highlights the selected index item", () => {
    const { rerender } = render(
      <SlashCommandPopup {...defaultProps} selectedIndex={0} />
    );

    // First item should have copper border (selected indicator)
    const clearButton = screen.getByText("/clear").closest("button");
    expect(clearButton?.className).toContain("border-l-ciab-copper");

    // Second item should have transparent border (not selected)
    const compactButton = screen.getByText("/compact").closest("button");
    expect(compactButton?.className).toContain("border-l-transparent");

    // Change selected index
    rerender(<SlashCommandPopup {...defaultProps} selectedIndex={1} />);
    const clearButton2 = screen.getByText("/clear").closest("button");
    const compactButton2 = screen.getByText("/compact").closest("button");
    expect(clearButton2?.className).toContain("border-l-transparent");
    expect(compactButton2?.className).toContain("border-l-ciab-copper");
  });

  it("groups commands by category in correct order", () => {
    render(<SlashCommandPopup {...defaultProps} />);

    const allText = document.body.textContent || "";
    const sessionIdx = allText.indexOf("Session");
    const agentIdx = allText.indexOf("Agent");
    const toolsIdx = allText.indexOf("Tools");
    const navIdx = allText.indexOf("Navigation");
    const helpIdx = allText.indexOf("Help");

    expect(sessionIdx).toBeLessThan(agentIdx);
    expect(agentIdx).toBeLessThan(toolsIdx);
    expect(toolsIdx).toBeLessThan(navIdx);
    expect(navIdx).toBeLessThan(helpIdx);
  });

  it("hides category headers when all commands in that category are filtered out", () => {
    render(<SlashCommandPopup {...defaultProps} filter="v" />);

    // Only "vim" matches, which is in "navigation"
    expect(screen.getByText("Navigation")).toBeInTheDocument();
    expect(screen.queryByText("Session")).not.toBeInTheDocument();
    expect(screen.queryByText("Agent")).not.toBeInTheDocument();
  });

  it("renders with empty commands array", () => {
    const { container } = render(
      <SlashCommandPopup {...defaultProps} commands={[]} />
    );

    expect(container.firstChild).toBeNull();
  });

  it("shows header with 'Slash Commands' when no filter", () => {
    render(<SlashCommandPopup {...defaultProps} />);

    expect(screen.getByText("Slash Commands")).toBeInTheDocument();
  });

  it("shows matching filter text in header when filtering", () => {
    render(<SlashCommandPopup {...defaultProps} filter="cl" />);

    expect(screen.getByText("Commands matching")).toBeInTheDocument();
    // The filter "/cl" appears in the header and footer
    const matches = screen.getAllByText(/\/cl/);
    expect(matches.length).toBeGreaterThanOrEqual(1);
  });

  it("shows command count in footer", () => {
    render(<SlashCommandPopup {...defaultProps} />);

    expect(screen.getByText(/6 commands/)).toBeInTheDocument();
  });

  it("shows arg names for commands with args", () => {
    render(<SlashCommandPopup {...defaultProps} />);

    // /model has an arg called "model"
    const argBadges = screen.getAllByText("model");
    // One is the command name, one is the arg badge
    expect(argBadges.length).toBeGreaterThanOrEqual(1);
  });
});
