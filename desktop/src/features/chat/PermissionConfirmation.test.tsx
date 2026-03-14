import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import PermissionConfirmation from "./PermissionConfirmation";

describe("PermissionConfirmation", () => {
  const defaultProps = {
    requestId: "req-123",
    toolName: "Bash",
    toolInput: { command: "ls -la" },
    riskLevel: "high" as const,
    onApprove: vi.fn(),
    onDeny: vi.fn(),
  };

  it("renders tool name and risk level", () => {
    render(<PermissionConfirmation {...defaultProps} />);

    expect(screen.getByText("Bash")).toBeInTheDocument();
    expect(screen.getByText("high")).toBeInTheDocument();
  });

  it("shows approve and deny buttons when not resolved", () => {
    render(<PermissionConfirmation {...defaultProps} />);

    expect(screen.getByText("Approve")).toBeInTheDocument();
    expect(screen.getByText("Deny")).toBeInTheDocument();
  });

  it("calls onApprove with request ID when approve is clicked", async () => {
    const user = userEvent.setup();
    const onApprove = vi.fn();

    render(
      <PermissionConfirmation {...defaultProps} onApprove={onApprove} />
    );

    await user.click(screen.getByText("Approve"));
    expect(onApprove).toHaveBeenCalledWith("req-123");
  });

  it("calls onDeny with request ID when deny is clicked", async () => {
    const user = userEvent.setup();
    const onDeny = vi.fn();

    render(
      <PermissionConfirmation {...defaultProps} onDeny={onDeny} />
    );

    await user.click(screen.getByText("Deny"));
    expect(onDeny).toHaveBeenCalledWith("req-123");
  });

  it("shows Approved badge when status is approved", () => {
    render(
      <PermissionConfirmation {...defaultProps} status="approved" />
    );

    expect(screen.getByText("Approved")).toBeInTheDocument();
    expect(screen.queryByText("Approve")).not.toBeInTheDocument();
    expect(screen.queryByText("Deny")).not.toBeInTheDocument();
  });

  it("shows Denied badge when status is denied", () => {
    render(
      <PermissionConfirmation {...defaultProps} status="denied" />
    );

    expect(screen.getByText("Denied")).toBeInTheDocument();
    expect(screen.queryByText("Approve")).not.toBeInTheDocument();
    expect(screen.queryByText("Deny")).not.toBeInTheDocument();
  });

  it("expands to show tool input on toggle click", async () => {
    const user = userEvent.setup();

    render(<PermissionConfirmation {...defaultProps} />);

    // Input should not be visible initially
    expect(screen.queryByText(/"command"/)).not.toBeInTheDocument();

    // Click the expand toggle (the button with chevron)
    const expandButtons = screen.getAllByRole("button");
    // The last non-action button is the expand toggle
    const expandButton = expandButtons.find(
      (btn) =>
        !btn.textContent?.includes("Approve") &&
        !btn.textContent?.includes("Deny")
    );
    expect(expandButton).toBeTruthy();
    await user.click(expandButton!);

    // Input JSON should now be visible
    expect(screen.getByText(/"command"/)).toBeInTheDocument();
  });

  it("renders different risk level styles", () => {
    const { rerender } = render(
      <PermissionConfirmation {...defaultProps} riskLevel="low" />
    );
    expect(screen.getByText("low")).toBeInTheDocument();

    rerender(
      <PermissionConfirmation {...defaultProps} riskLevel="medium" />
    );
    expect(screen.getByText("medium")).toBeInTheDocument();
  });

  it("handles unknown tool names gracefully", () => {
    render(
      <PermissionConfirmation
        {...defaultProps}
        toolName="CustomTool"
        riskLevel="medium"
      />
    );

    expect(screen.getByText("CustomTool")).toBeInTheDocument();
  });
});
