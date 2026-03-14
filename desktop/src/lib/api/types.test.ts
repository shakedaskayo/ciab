import { describe, it, expect } from "vitest";
import type {
  PermissionMode,
  RiskLevel,
  PermissionRequestData,
  PermissionResponseData,
  StreamEventType,
  SlashCommand,
  SlashCommandCategory,
} from "./types";

describe("Permission types", () => {
  it("PermissionMode has all expected values", () => {
    const modes: PermissionMode[] = [
      "auto_approve",
      "approve_edits",
      "approve_all",
      "plan_only",
    ];
    expect(modes).toHaveLength(4);
  });

  it("RiskLevel has all expected values", () => {
    const levels: RiskLevel[] = ["low", "medium", "high"];
    expect(levels).toHaveLength(3);
  });

  it("PermissionRequestData has required fields", () => {
    const data: PermissionRequestData = {
      request_id: "abc-123",
      tool_name: "Bash",
      tool_input: { command: "ls" },
      risk_level: "high",
    };
    expect(data.request_id).toBe("abc-123");
    expect(data.tool_name).toBe("Bash");
    expect(data.risk_level).toBe("high");
  });

  it("PermissionResponseData has required fields", () => {
    const data: PermissionResponseData = {
      request_id: "abc-123",
      tool_name: "Bash",
      approved: true,
    };
    expect(data.approved).toBe(true);
  });

  it("StreamEventType includes permission events", () => {
    const types: StreamEventType[] = [
      "permission_request",
      "permission_response",
    ];
    expect(types).toContain("permission_request");
    expect(types).toContain("permission_response");
  });
});

describe("SlashCommand types", () => {
  it("should accept valid SlashCommand objects", () => {
    const cmd: SlashCommand = {
      name: "clear",
      description: "Clear conversation",
      category: "session",
      args: [],
      provider_native: false,
    };
    expect(cmd.name).toBe("clear");
    expect(cmd.category).toBe("session");
    expect(cmd.provider_native).toBe(false);
  });

  it("should accept SlashCommand with args", () => {
    const cmd: SlashCommand = {
      name: "model",
      description: "Switch model",
      category: "agent",
      args: [{ name: "model", description: "Model name", required: false }],
      provider_native: true,
    };
    expect(cmd.args).toHaveLength(1);
    expect(cmd.args[0].name).toBe("model");
    expect(cmd.args[0].required).toBe(false);
  });

  it("should accept all valid categories", () => {
    const categories: SlashCommandCategory[] = [
      "session",
      "agent",
      "tools",
      "navigation",
      "help",
    ];
    expect(categories).toHaveLength(5);
  });
});
