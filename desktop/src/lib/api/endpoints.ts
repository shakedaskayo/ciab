import { get, post, put, del, putRaw, getRaw } from "./client";
import type {
  SandboxInfo,
  SandboxSpec,
  Session,
  SessionWithMessages,
  Message,
  ExecRequest,
  ExecResult,
  FileInfo,
  ResourceStats,
  CredentialSet,
  CreateCredentialRequest,
  Workspace,
  CreateWorkspaceRequest,
  UpdateWorkspaceRequest,
  TemplateSource,
  CreateFromTemplateRequest,
  AddTemplateSourceRequest,
  GatewayStatus,
  GatewayTunnel,
  GatewayConfigResponse,
  UpdateGatewayConfigRequest,
  ClientToken,
  CreateTokenRequest,
  CreateTokenResponse,
  CreateTunnelRequest,
  ExposeRequest,
  ExposeResponse,
  DiscoverResponse,
  ProviderPrepareResult,
  SlashCommand,
  QueuedMessage,
  Channel,
  ChannelMessage,
  CreateChannelRequest,
  UpdateChannelRequest,
} from "./types";

// --- Sandboxes ---

export interface CreateSandboxResponse {
  sandbox_id: string;
  status: string;
}

export const sandboxes = {
  create: (spec: SandboxSpec) =>
    post<CreateSandboxResponse>("/api/v1/sandboxes", spec),

  list: (params?: { state?: string; provider?: string }) => {
    const query = new URLSearchParams();
    if (params?.state) query.set("state", params.state);
    if (params?.provider) query.set("provider", params.provider);
    const qs = query.toString();
    return get<SandboxInfo[]>(`/api/v1/sandboxes${qs ? `?${qs}` : ""}`);
  },

  get: (id: string) => get<SandboxInfo>(`/api/v1/sandboxes/${id}`),

  delete: (id: string) => del<void>(`/api/v1/sandboxes/${id}`),

  start: (id: string) => post<void>(`/api/v1/sandboxes/${id}/start`),

  stop: (id: string) => post<void>(`/api/v1/sandboxes/${id}/stop`),

  pause: (id: string) => post<void>(`/api/v1/sandboxes/${id}/pause`),

  resume: (id: string) => post<void>(`/api/v1/sandboxes/${id}/resume`),

  stats: (id: string) => get<ResourceStats>(`/api/v1/sandboxes/${id}/stats`),

  logs: (id: string, params?: { follow?: boolean; tail?: number }) => {
    const query = new URLSearchParams();
    if (params?.follow) query.set("follow", "true");
    if (params?.tail) query.set("tail", params.tail.toString());
    const qs = query.toString();
    return get<string>(`/api/v1/sandboxes/${id}/logs${qs ? `?${qs}` : ""}`);
  },
};

// --- Sessions ---

export const sessions = {
  create: (sandboxId: string, metadata?: Record<string, unknown>) =>
    post<Session>(`/api/v1/sandboxes/${sandboxId}/sessions`, { metadata }),

  list: (sandboxId: string) =>
    get<Session[]>(`/api/v1/sandboxes/${sandboxId}/sessions`),

  get: (sessionId: string) =>
    get<SessionWithMessages>(`/api/v1/sessions/${sessionId}`),

  sendMessage: (sessionId: string, content: { role: string; content: Array<{ type: string; text: string }> }) =>
    post<Message>(`/api/v1/sessions/${sessionId}/messages`, content),

  interrupt: (sessionId: string) =>
    post<void>(`/api/v1/sessions/${sessionId}/interrupt`),

  setPermissionMode: (
    sessionId: string,
    body: {
      mode: string;
      always_require_approval?: string[];
      always_allow?: string[];
    }
  ) => post<{ status: string; mode: string }>(`/api/v1/sessions/${sessionId}/permissions`, body),

  respondToPermission: (sessionId: string, requestId: string, approved: boolean) =>
    post<{ status: string; approved: boolean }>(
      `/api/v1/sessions/${sessionId}/permissions/${requestId}/respond`,
      { approved }
    ),

  respondToUserInput: (sessionId: string, requestId: string, answer: string) =>
    post<{ status: string }>(
      `/api/v1/sessions/${sessionId}/user-input/${requestId}/respond`,
      { answer }
    ),

  updateSkills: (sessionId: string, activeSkills: Array<{ source: string; skill_id?: string; name?: string }>) =>
    post<{ status: string; active_skills: unknown[] }>(
      `/api/v1/sessions/${sessionId}/skills`,
      { active_skills: activeSkills }
    ),

  getQueue: (sessionId: string) =>
    get<{ messages: QueuedMessage[]; processing: boolean; queue_length: number }>(
      `/api/v1/sessions/${sessionId}/queue`
    ),

  cancelQueuedMessage: (sessionId: string, messageId: string) =>
    del<{ status: string; queue_length: number }>(
      `/api/v1/sessions/${sessionId}/queue/${messageId}`
    ),
};

// --- Execution ---

export const exec = {
  run: (sandboxId: string, request: ExecRequest) =>
    post<ExecResult>(`/api/v1/sandboxes/${sandboxId}/exec`, request),
};

// --- Files ---

export const files = {
  list: (sandboxId: string, path: string = "/") => {
    const query = new URLSearchParams({ path });
    return get<FileInfo[]>(`/api/v1/sandboxes/${sandboxId}/files?${query}`);
  },

  download: (sandboxId: string, filePath: string) =>
    getRaw(`/api/v1/sandboxes/${sandboxId}/files/${filePath}`),

  upload: (sandboxId: string, filePath: string, data: ArrayBuffer) =>
    putRaw(`/api/v1/sandboxes/${sandboxId}/files/${filePath}`, data),

  delete: (sandboxId: string, filePath: string) =>
    del<void>(`/api/v1/sandboxes/${sandboxId}/files/${filePath}`),
};

// --- Credentials ---

export const credentials = {
  create: (request: CreateCredentialRequest) =>
    post<CredentialSet>("/api/v1/credentials", request),

  list: () => get<CredentialSet[]>("/api/v1/credentials"),

  get: (id: string) => get<CredentialSet>(`/api/v1/credentials/${id}`),

  delete: (id: string) => del<void>(`/api/v1/credentials/${id}`),
};

// --- Workspaces ---

export const workspaces = {
  create: (request: CreateWorkspaceRequest) =>
    post<Workspace>("/api/v1/workspaces", request),

  list: (params?: { name?: string }) => {
    const query = new URLSearchParams();
    if (params?.name) query.set("name", params.name);
    const qs = query.toString();
    return get<Workspace[]>(`/api/v1/workspaces${qs ? `?${qs}` : ""}`);
  },

  get: (id: string) => get<Workspace>(`/api/v1/workspaces/${id}`),

  update: (id: string, request: UpdateWorkspaceRequest) =>
    put<Workspace>(`/api/v1/workspaces/${id}`, request),

  delete: (id: string) => del<void>(`/api/v1/workspaces/${id}`),

  launch: (id: string) =>
    post<{ sandbox_id: string; workspace_id: string; status: string }>(
      `/api/v1/workspaces/${id}/launch`
    ),

  sandboxes: (id: string) =>
    get<{ sandbox_ids: string[] }>(`/api/v1/workspaces/${id}/sandboxes`),

  exportToml: async (id: string): Promise<string> => {
    const { getServerUrl, getApiKey } = await import("../stores/connection-store");
    const url = `${getServerUrl()}/api/v1/workspaces/${id}/export`;
    const headers: Record<string, string> = {};
    const key = getApiKey();
    if (key) headers["Authorization"] = `Bearer ${key}`;
    const resp = await fetch(url, { headers });
    if (!resp.ok) throw new Error(`Export failed: ${resp.status}`);
    return resp.text();
  },

  importToml: async (toml: string): Promise<Workspace> => {
    const { getServerUrl, getApiKey } = await import("../stores/connection-store");
    const url = `${getServerUrl()}/api/v1/workspaces/import`;
    const headers: Record<string, string> = { "Content-Type": "text/plain" };
    const key = getApiKey();
    if (key) headers["Authorization"] = `Bearer ${key}`;
    const resp = await fetch(url, { method: "POST", headers, body: toml });
    if (!resp.ok) throw new Error(`Import failed: ${resp.status}`);
    return resp.json();
  },
};

// --- Templates ---

export const templates = {
  list: () => get<Workspace[]>("/api/v1/templates"),

  create: (request: CreateWorkspaceRequest) =>
    post<Workspace>("/api/v1/templates", request),

  createFromTemplate: (templateId: string, request: CreateFromTemplateRequest) =>
    post<Workspace>(`/api/v1/templates/${templateId}/create`, request),

  listSources: () => get<TemplateSource[]>("/api/v1/templates/sources"),

  addSource: (request: AddTemplateSourceRequest) =>
    post<TemplateSource>("/api/v1/templates/sources", request),

  deleteSource: (id: string) =>
    del<void>(`/api/v1/templates/sources/${id}`),

  syncSource: (id: string) =>
    post<{ synced: number }>(`/api/v1/templates/sources/${id}/sync`),
};

// --- Skills (registry proxy) ---

export interface SkillSearchResult {
  id: string;
  skillId: string;
  name: string;
  installs: number;
  source: string;
}

export interface SkillSearchResponse {
  query: string;
  skills: SkillSearchResult[];
}

export interface RepoSkillEntry {
  path: string;
  skill_id: string;
}

export interface SkillMetadataResponse {
  source: string;
  name: string | null;
  description: string | null;
  raw_content: string;
  available_skills: RepoSkillEntry[];
}

export const skills = {
  search: (q: string, limit = 20) =>
    get<SkillSearchResponse>(
      `/api/v1/skills/search?q=${encodeURIComponent(q)}&limit=${limit}`
    ),

  trending: () => get<SkillSearchResponse>("/api/v1/skills/trending"),

  metadata: (source: string, skillId?: string, ref?: string) => {
    const query = new URLSearchParams({ source });
    if (skillId) query.set("skill_id", skillId);
    if (ref) query.set("ref", ref);
    return get<SkillMetadataResponse>(`/api/v1/skills/metadata?${query}`);
  },
};

// --- Gateway ---

export const gateway = {
  status: () => get<GatewayStatus>("/api/v1/gateway/status"),

  getConfig: () => get<GatewayConfigResponse>("/api/v1/gateway/config"),

  updateConfig: (config: UpdateGatewayConfigRequest) =>
    put<{ status: string; config: GatewayConfigResponse }>("/api/v1/gateway/config", config),

  discover: () => get<DiscoverResponse>("/api/v1/gateway/discover"),

  createToken: (request: CreateTokenRequest) =>
    post<CreateTokenResponse>("/api/v1/gateway/tokens", request),

  listTokens: () => get<ClientToken[]>("/api/v1/gateway/tokens"),

  getToken: (id: string) => get<ClientToken>(`/api/v1/gateway/tokens/${id}`),

  revokeToken: (id: string) =>
    del<{ status: string }>(`/api/v1/gateway/tokens/${id}`),

  createTunnel: (request: CreateTunnelRequest) =>
    post<GatewayTunnel>("/api/v1/gateway/tunnels", request),

  listTunnels: () => get<GatewayTunnel[]>("/api/v1/gateway/tunnels"),

  getTunnel: (id: string) =>
    get<GatewayTunnel>(`/api/v1/gateway/tunnels/${id}`),

  deleteTunnel: (id: string) =>
    del<{ status: string }>(`/api/v1/gateway/tunnels/${id}`),

  createSandboxTunnel: (sandboxId: string) =>
    post<GatewayTunnel>(`/api/v1/gateway/tunnels/sandbox/${sandboxId}`),

  expose: (request: ExposeRequest) =>
    post<ExposeResponse>("/api/v1/gateway/expose", request),

  prepareProvider: (provider: string) =>
    post<ProviderPrepareResult>("/api/v1/gateway/providers/prepare", { provider }),
};

// --- Channels ---

export const channels = {
  create: (request: CreateChannelRequest) =>
    post<Channel>("/api/v1/channels", request),

  list: (params?: { provider?: string; state?: string; name?: string }) => {
    const query = new URLSearchParams();
    if (params?.provider) query.set("provider", params.provider);
    if (params?.state) query.set("state", params.state);
    if (params?.name) query.set("name", params.name);
    const qs = query.toString();
    return get<Channel[]>(`/api/v1/channels${qs ? `?${qs}` : ""}`);
  },

  get: (id: string) => get<Channel>(`/api/v1/channels/${id}`),

  update: (id: string, request: UpdateChannelRequest) =>
    put<Channel>(`/api/v1/channels/${id}`, request),

  delete: (id: string) => del<void>(`/api/v1/channels/${id}`),

  start: (id: string) =>
    post<{ status: string }>(`/api/v1/channels/${id}/start`),

  stop: (id: string) =>
    post<{ status: string }>(`/api/v1/channels/${id}/stop`),

  restart: (id: string) =>
    post<{ status: string }>(`/api/v1/channels/${id}/restart`),

  status: (id: string) =>
    get<{ state: string }>(`/api/v1/channels/${id}/status`),

  qr: (id: string) =>
    get<{ qr_code: string | null }>(`/api/v1/channels/${id}/qr`),

  messages: (id: string, params?: { limit?: number; sender_id?: string }) => {
    const query = new URLSearchParams();
    if (params?.limit) query.set("limit", params.limit.toString());
    if (params?.sender_id) query.set("sender_id", params.sender_id);
    const qs = query.toString();
    return get<ChannelMessage[]>(
      `/api/v1/channels/${id}/messages${qs ? `?${qs}` : ""}`
    );
  },
};

// --- Agents ---

export const agents = {
  list: () => get<string[]>("/api/v1/agents"),
  getCommands: (provider: string) =>
    get<SlashCommand[]>(`/api/v1/agents/${provider}/commands`),
};

// --- Health ---

interface HealthResponse {
  status: string;
}

export const health = {
  /** Checks /health and verifies the response is from a real CIAB server. */
  check: async (): Promise<void> => {
    const res = await get<HealthResponse>("/health");
    // The CIAB server returns {"status":"healthy"}.
    // Other services on the same port may return something else (e.g. bare `true`).
    if (!res || typeof res !== "object" || res.status !== "healthy") {
      throw new Error(
        "Server responded but does not appear to be a CIAB instance"
      );
    }
  },
  ready: () => get<HealthResponse>("/ready"),
};
