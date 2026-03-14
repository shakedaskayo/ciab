import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import PermissionModeSelector from "./PermissionModeSelector";

describe("PermissionModeSelector", () => {
  it("renders all four mode buttons", () => {
    render(
      <PermissionModeSelector mode="auto_approve" onChange={vi.fn()} />
    );

    expect(screen.getByText("Auto")).toBeInTheDocument();
    expect(screen.getByText("Safe")).toBeInTheDocument();
    expect(screen.getByText("Strict")).toBeInTheDocument();
    expect(screen.getByText("Plan")).toBeInTheDocument();
  });

  it("calls onChange with the clicked mode", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <PermissionModeSelector mode="auto_approve" onChange={onChange} />
    );

    await user.click(screen.getByText("Strict"));
    expect(onChange).toHaveBeenCalledWith("approve_all");
  });

  it("calls onChange for each mode", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <PermissionModeSelector mode="auto_approve" onChange={onChange} />
    );

    await user.click(screen.getByText("Safe"));
    expect(onChange).toHaveBeenCalledWith("approve_edits");

    await user.click(screen.getByText("Plan"));
    expect(onChange).toHaveBeenCalledWith("plan_only");

    await user.click(screen.getByText("Auto"));
    expect(onChange).toHaveBeenCalledWith("auto_approve");
  });

  it("shows tooltips with mode descriptions", () => {
    render(
      <PermissionModeSelector mode="approve_edits" onChange={vi.fn()} />
    );

    expect(screen.getByTitle("Auto-approve all tool calls")).toBeInTheDocument();
    expect(screen.getByTitle("Approve edits & commands only")).toBeInTheDocument();
    expect(screen.getByTitle("Approve every tool call")).toBeInTheDocument();
    expect(screen.getByTitle("Read-only planning mode")).toBeInTheDocument();
  });
});
