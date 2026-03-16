// Mirrors ciab-core types

export type SandboxState =
  | "pending"
  | "creating"
  | "running"
  | "pausing"
  | "paused"
  | "stopping"
  | "stopped"
  | "terminated"
  | "failed";

export type SessionState =
  | "active"
  | "waiting_for_input"
  | "processing"
  | "completed"
  | "failed";

export type MessageRole = "user" | "assistant" | "system" | "tool";

export type CredentialType =
  | "api_key"
  | "env_vars"
  | "git_token"
  | "oauth_token"
  | "ssh_key"
  | "file";

export type SandboxPersistence = "ephemeral" | "persistent";

export type StreamEventType =
  | "connected"
  | "reconnect"
  | "keepalive"
  | "text_delta"
  | "text_complete"
  | "thinking_delta"
  | "subagent_start"
  | "subagent_end"
  | "tool_use_start"
  | "tool_input_delta"
  | "tool_use_complete"
  | "tool_result"
  | "sandbox_state_changed"
  | "provisioning_step"
  | "provisioning_complete"
  | "provisioning_failed"
  | "session_created"
  | "session_completed"
  | "session_failed"
  | "permission_request"
  | "permission_response"
  | "error"
  | "stats"
  | "log_line"
  | "user_input_request"
  | "tool_progress"
  | "result_error"
  | "queue_updated"
  | "file_changed";

// --- Queue ---

export interface QueuedMessage {
  id: string;
  session_id: string;
  prompt_text: string;
  queued_at: string;
}

// --- Permissions ---

export type PermissionMode =
  | "auto_approve"
  | "approve_edits"
  | "approve_all"
  | "plan_only"
  | "unrestricted";

export type RiskLevel = "low" | "medium" | "high";

export interface PermissionRequestData {
  request_id: string;
  tool_name: string;
  tool_input: unknown;
  risk_level: RiskLevel;
}

export interface PermissionResponseData {
  request_id: string;
  tool_name: string;
  approved: boolean;
}

export interface UserInputQuestion {
  header?: string;
  question: string;
  multiSelect?: boolean;
  options?: Array<{ label: string; description?: string }>;
}

export interface UserInputRequestData {
  tool_use_id: string;
  request_id?: string;
  questions: UserInputQuestion[];
}

// --- Core Types ---

export interface SandboxInfo {
  id: string;
  name: string | null;
  state: SandboxState;
  persistence: SandboxPersistence;
  agent_provider: string;
  endpoint_url: string | null;
  resource_stats: ResourceStats | null;
  labels: Record<string, string>;
  created_at: string;
  updated_at: string;
  spec: SandboxSpec;
}

export interface SandboxSpec {
  name?: string;
  agent_provider: string;
  image?: string;
  resource_limits?: ResourceLimits;
  persistence?: SandboxPersistence;
  network?: NetworkSpec;
  env_vars?: Record<string, string>;
  volumes?: VolumeMount[];
  ports?: PortMapping[];
  git_repos?: GitRepoSpec[];
  local_mounts?: LocalMountSpec[];
  credentials?: string[];
  provisioning_scripts?: string[];
  labels?: Record<string, string>;
  timeout_secs?: number;
  agent_config?: AgentConfig;
}

export interface ResourceLimits {
  cpu_cores?: number;
  memory_mb?: number;
  disk_mb?: number;
  max_processes?: number;
}

export interface ResourceStats {
  cpu_usage_percent: number;
  memory_used_mb: number;
  memory_limit_mb: number;
  disk_used_mb: number;
  disk_limit_mb: number;
  network_rx_bytes: number;
  network_tx_bytes: number;
}

export interface NetworkSpec {
  enabled: boolean;
  allowed_hosts?: string[];
  dns_servers?: string[];
}

export interface VolumeMount {
  host_path: string;
  container_path: string;
  read_only?: boolean;
}

export interface PortMapping {
  host_port: number;
  container_port: number;
  protocol?: string;
}

export interface GitRepoSpec {
  url: string;
  branch?: string;
  tag?: string;
  commit?: string;
  dest_path?: string;
  credential_id?: string;
  depth?: number;
  sparse_paths?: string[];
  submodules?: boolean;
}

export type SyncMode = "copy" | "link" | "bind";

export interface LocalMount {
  source: string;
  dest_path?: string;
  sync_mode?: SyncMode;
  exclude_patterns?: string[];
  writeback?: boolean;
  watch?: boolean;
}

export interface LocalMountSpec {
  source: string;
  dest_path: string;
  sync_mode: string;
  exclude_patterns: string[];
  writeback: boolean;
}

export interface AgentConfig {
  provider: string;
  model?: string;
  system_prompt?: string;
  max_tokens?: number;
  temperature?: number;
  tools_enabled?: boolean;
  mcp_servers?: McpServerConfig[];
  allowed_tools?: string[];
  denied_tools?: string[];
  extra?: Record<string, unknown>;
}

export interface McpServerConfig {
  name: string;
  url: string;
  api_key?: string;
}

// --- Sessions ---

export interface Session {
  id: string;
  sandbox_id: string;
  state: SessionState;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface SessionWithMessages extends Session {
  messages: Message[];
}

export interface Message {
  id: string;
  session_id: string;
  role: MessageRole;
  content: MessageContent[];
  timestamp: string;
}

export type MessageContent =
  | { type: "text"; text: string }
  | { type: "thinking"; thinking: string }
  | { type: "tool_use"; id: string; name: string; input: unknown; agent_name?: string }
  | {
      type: "tool_result";
      tool_use_id: string;
      content: string;
      is_error?: boolean;
    }
  | { type: "image"; media_type: string; data: string };

// --- Execution ---

export interface ExecRequest {
  command: string[];
  workdir?: string;
  env?: Record<string, string>;
  stdin?: string;
  timeout_secs?: number;
  tty?: boolean;
}

export interface ExecResult {
  exit_code: number;
  stdout: string;
  stderr: string;
  duration_ms: number;
}

// --- Files ---

export interface FileInfo {
  path: string;
  size: number;
  is_dir: boolean;
  mode: number;
  modified_at: string | null;
}

// --- Credentials ---

export interface CredentialSet {
  id: string;
  name: string;
  credential_type: CredentialType;
  labels: Record<string, string>;
  created_at: string;
  expires_at: string | null;
}

export interface CreateCredentialRequest {
  name: string;
  credential_type: CredentialType;
  /** The secret value to store (will be encrypted server-side). */
  value: string;
  labels?: Record<string, string>;
  expires_at?: string;
}

// --- Streaming ---

export interface StreamEvent {
  id: string;
  sandbox_id: string;
  session_id: string | null;
  event_type: StreamEventType;
  data: unknown;
  timestamp: string;
}

// --- Runtime ---

export type RuntimeBackend = "default" | "local" | "opensandbox" | "docker" | "kubernetes";

export interface KubernetesToleration {
  key: string;
  operator: string;
  value?: string;
  effect?: string;
}

export interface KubernetesRuntimeConfig {
  namespace?: string;
  agent_image?: string;
  /** RuntimeClass for microvm isolation (e.g. "kata-containers", "kata-qemu"). */
  runtime_class?: string;
  node_selector?: Record<string, string>;
  tolerations?: KubernetesToleration[];
  image_pull_secrets?: string[];
  storage_class?: string;
  workspace_pvc_size?: string;
  service_account?: string;
  create_network_policy?: boolean;
  run_as_non_root?: boolean;
  drop_all_capabilities?: boolean;
  default_cpu_request?: string;
  default_cpu_limit?: string;
  default_memory_request?: string;
  default_memory_limit?: string;
}

export interface WorkspaceRuntimeConfig {
  backend?: RuntimeBackend;
  local_workdir?: string;
  kubernetes_namespace?: string;
  kubernetes_runtime_class?: string;
  kubernetes_node_selector?: Record<string, string>;
  kubernetes_image?: string;
}

export type GitCloneStrategy = "clone" | "worktree";

export interface AgentFsConfig {
  enabled?: boolean;
  binary?: string;
  db_path?: string;
  operation_logging?: boolean;
}

// --- Workspaces ---

export interface Workspace {
  id: string;
  name: string;
  description: string | null;
  spec: WorkspaceSpec;
  labels: Record<string, string>;
  created_at: string;
  updated_at: string;
}

export interface WorkspaceSpec {
  name?: string;
  description?: string;
  repositories?: WorkspaceRepo[];
  local_mounts?: LocalMount[];
  skills?: WorkspaceSkill[];
  pre_commands?: PreCommand[];
  binaries?: BinaryInstall[];
  filesystem?: FilesystemConfig;
  agent?: WorkspaceAgentConfig;
  subagents?: SubagentConfig[];
  credentials?: WorkspaceCredential[];
  env_vars?: Record<string, string>;
  env_file?: string;
  resource_limits?: ResourceLimits;
  network?: NetworkSpec;
  volumes?: VolumeMount[];
  ports?: PortMapping[];
  labels?: Record<string, string>;
  timeout_secs?: number;
  image?: string;
  runtime?: WorkspaceRuntimeConfig;
}

export interface WorkspaceRepo {
  url: string;
  branch?: string;
  tag?: string;
  commit?: string;
  dest_path?: string;
  depth?: number;
  credential_id?: string;
  sparse_paths?: string[];
  submodules?: boolean;
  strategy?: GitCloneStrategy;
}

export interface WorkspaceSkill {
  source: string;
  version?: string;
  name?: string;
  enabled?: boolean;
  config?: Record<string, unknown>;
}

export interface PreCommand {
  name?: string;
  command: string;
  args?: string[];
  workdir?: string;
  env?: Record<string, string>;
  fail_on_error?: boolean;
  timeout_secs?: number;
}

export interface BinaryInstall {
  name: string;
  method?: "apt" | "cargo" | "npm" | "pip" | "custom";
  version?: string;
  install_command?: string;
}

export interface FilesystemConfig {
  workdir?: string;
  cow_isolation?: boolean;
  readonly_paths?: string[];
  writable_paths?: string[];
  tmp_size_mb?: number;
  persist_changes?: boolean;
  max_file_size_bytes?: number;
  exclude_patterns?: string[];
  agentfs?: AgentFsConfig;
}

export interface WorkspaceAgentConfig {
  provider: string;
  model?: string;
  system_prompt?: string;
  max_tokens?: number;
  temperature?: number;
  tools_enabled?: boolean;
  mcp_servers?: McpServerConfig[];
  allowed_tools?: string[];
  denied_tools?: string[];
  extra?: Record<string, unknown>;
}

export interface SubagentConfig {
  name: string;
  provider: string;
  model?: string;
  system_prompt?: string;
  activation?: "always" | "on_demand" | { on_event: { events: string[] } };
  allowed_tools?: string[];
  mcp_servers?: McpServerConfig[];
  extra?: Record<string, unknown>;
}

export interface WorkspaceCredential {
  id?: string;
  name?: string;
  vault_provider?: string;
  vault_path?: string;
  env_var?: string;
  file_path?: string;
}

export interface CreateWorkspaceRequest {
  name: string;
  description?: string;
  spec: WorkspaceSpec;
  labels?: Record<string, string>;
}

export interface UpdateWorkspaceRequest {
  name?: string;
  description?: string;
  spec?: WorkspaceSpec;
  labels?: Record<string, string>;
}

// --- Templates ---

export interface TemplateSource {
  id: string;
  name: string;
  url: string;
  branch: string;
  templates_path: string;
  last_synced_at: string | null;
  template_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateFromTemplateRequest {
  name: string;
  description?: string;
  overrides?: Partial<WorkspaceSpec>;
}

export interface AddTemplateSourceRequest {
  name: string;
  url: string;
  branch?: string;
  templates_path?: string;
}

// --- Gateway ---

export type TunnelType = "frp" | "bore" | "cloudflare" | "ngrok" | "lan" | "manual";
export type TunnelProvider = "frp" | "bore" | "cloudflare" | "ngrok";
export type TunnelState = "active" | "stopped" | "error";

export interface GatewayTunnel {
  id: string;
  sandbox_id: string | null;
  tunnel_type: TunnelType;
  public_url: string;
  local_port: number;
  state: TunnelState;
  config_json: Record<string, unknown>;
  error_message: string | null;
  created_at: string;
  updated_at: string;
}

export type TokenScopeType =
  | "full_access"
  | "sandbox_access"
  | "workspace_access"
  | "read_only"
  | "chat_only";

export interface TokenScope {
  type: TokenScopeType;
  sandbox_id?: string;
  workspace_id?: string;
}

export interface ClientToken {
  id: string;
  name: string;
  token_hash: string;
  scopes: TokenScope[];
  expires_at: string | null;
  last_used_at: string | null;
  created_at: string;
  revoked_at: string | null;
}

export interface LanStatus {
  enabled: boolean;
  mdns_name: string | null;
  local_addresses: string[];
  advertise_port: number;
}

export interface FrpStatus {
  enabled: boolean;
  process_running: boolean;
  server_addr: string | null;
  proxy_count: number;
}

export interface TunnelProviderInfo {
  name: string;
  enabled: boolean;
  installed: boolean;
  binary_path: string | null;
  version: string | null;
  process_running: boolean;
  tunnel_count: number;
}

export interface ProviderPrepareResult {
  provider: string;
  installed: boolean;
  binary_path: string;
  version: string | null;
  message: string;
}

export interface GatewayStatus {
  enabled: boolean;
  active_provider: string;
  lan: LanStatus;
  providers: TunnelProviderInfo[];
  frp: FrpStatus;
  active_tunnels: number;
  active_tokens: number;
}

export interface CreateTokenRequest {
  name: string;
  scopes?: TokenScope[];
  expires_secs?: number;
}

export interface CreateTokenResponse {
  token: string;
  token_info: ClientToken;
}

export interface CreateTunnelRequest {
  sandbox_id?: string;
  tunnel_type?: string;
  local_port?: number;
  public_url?: string;
}

export interface ExposeRequest {
  sandbox_id: string;
  token_name?: string;
  expires_secs?: number;
  scope?: TokenScope;
}

export interface ExposeResponse {
  tunnel: GatewayTunnel;
  token: string;
  token_info: ClientToken;
}

export interface DiscoverResponse {
  lan: LanStatus;
  server_version: string;
}

// --- Gateway Config (runtime update) ---

export interface GatewayConfigResponse {
  enabled: boolean;
  tunnel_provider: string;
  lan: {
    enabled: boolean;
    mdns_name: string;
    advertise_port: number;
  };
  frp: {
    enabled: boolean;
    frpc_binary: string;
    server_addr: string | null;
    server_port: number | null;
    auth_token: string | null;
    subdomain_prefix: string | null;
    tls_enable: boolean;
    config_template: string | null;
  };
  bore: {
    enabled: boolean;
    binary: string;
    server: string;
    server_port: number | null;
    secret: string | null;
    auto_install: boolean;
  };
  cloudflare: {
    enabled: boolean;
    binary: string;
    tunnel_token: string | null;
    tunnel_name: string | null;
    auto_install: boolean;
  };
  ngrok: {
    enabled: boolean;
    binary: string;
    authtoken: string | null;
    domain: string | null;
    region: string | null;
    auto_install: boolean;
  };
  routing: {
    mode: string;
    base_domain: string | null;
  };
  advanced: {
    custom_dns_cname: string | null;
    k8s_ingress_class: string | null;
    k8s_ingress_annotations: Record<string, string>;
  };
}

export interface UpdateGatewayConfigRequest {
  enabled?: boolean;
  tunnel_provider?: string;
  lan?: {
    enabled?: boolean;
    mdns_name?: string;
    advertise_port?: number;
  };
  frp?: {
    enabled?: boolean;
    server_addr?: string;
    server_port?: number;
    auth_token?: string;
    subdomain_prefix?: string;
    tls_enable?: boolean;
  };
  bore?: {
    enabled?: boolean;
    binary?: string;
    server?: string;
    server_port?: number;
    secret?: string;
    auto_install?: boolean;
  };
  cloudflare?: {
    enabled?: boolean;
    binary?: string;
    tunnel_token?: string;
    tunnel_name?: string;
    auto_install?: boolean;
  };
  ngrok?: {
    enabled?: boolean;
    binary?: string;
    authtoken?: string;
    domain?: string;
    region?: string;
    auto_install?: boolean;
  };
  routing?: {
    mode?: string;
    base_domain?: string;
  };
  advanced?: {
    custom_dns_cname?: string;
    k8s_ingress_class?: string;
    k8s_ingress_annotations?: Record<string, string>;
  };
}

// --- Slash Commands ---

export type SlashCommandCategory = "session" | "agent" | "tools" | "navigation" | "help";

export interface SlashCommandArg {
  name: string;
  description: string;
  required: boolean;
}

export interface SlashCommand {
  name: string;
  description: string;
  category: SlashCommandCategory;
  args: SlashCommandArg[];
  provider_native: boolean;
}

// --- Channels ---

export type ChannelProvider = "whatsapp" | "slack" | "webhook";

export type ChannelState =
  | "inactive"
  | "pairing"
  | "connected"
  | "reconnecting"
  | "failed"
  | "stopped";

export type MessageDirection = "inbound" | "outbound";

export type DmPolicy = "respond" | "allowed_only" | "ignore";

export type GroupPolicy = "all" | "mention_only" | "commands_only" | "ignore";

export interface ChannelBinding {
  type: "static" | "auto_provision";
  sandbox_id?: string;
  workspace_id?: string;
  ttl_secs?: number;
  persist_on_expire?: boolean;
}

export interface ChannelRules {
  allowed_senders?: string[];
  blocked_senders?: string[];
  reset_trigger?: string;
  dm_policy?: DmPolicy;
  group_policy?: GroupPolicy;
  rate_limit_per_minute?: number;
  persist_conversation?: boolean;
  max_message_length?: number;
}

export interface ChannelProviderConfig {
  provider: "whatsapp" | "slack" | "webhook";
  // WhatsApp
  session_dir?: string;
  phone_number?: string;
  // Slack
  bot_token?: string;
  app_token?: string;
  signing_secret?: string;
  listen_channels?: string[];
  // Webhook
  inbound_secret?: string;
  outbound_url?: string;
  outbound_headers?: Record<string, string>;
}

export interface Channel {
  id: string;
  name: string;
  description: string | null;
  provider: ChannelProvider;
  state: ChannelState;
  binding: ChannelBinding;
  provider_config: ChannelProviderConfig;
  rules: ChannelRules;
  labels: Record<string, string>;
  error_message: string | null;
  qr_code: string | null;
  created_at: string;
  updated_at: string;
}

export interface ChannelMessage {
  id: string;
  channel_id: string;
  direction: MessageDirection;
  sender_id: string;
  sender_name: string | null;
  sandbox_id: string | null;
  session_id: string | null;
  content: string;
  platform_metadata: Record<string, unknown>;
  timestamp: string;
}

export interface CreateChannelRequest {
  name: string;
  description?: string;
  provider: ChannelProvider;
  binding: ChannelBinding;
  provider_config: ChannelProviderConfig;
  rules?: ChannelRules;
  labels?: Record<string, string>;
}

export interface UpdateChannelRequest {
  name?: string;
  description?: string;
  binding?: ChannelBinding;
  provider_config?: ChannelProviderConfig;
  rules?: ChannelRules;
  labels?: Record<string, string>;
}

// --- LLM Providers ---

export type LlmProviderKind = "anthropic" | "openai" | "google" | "ollama" | "openrouter" | "custom";

export interface LlmProvider {
  id: string;
  name: string;
  kind: LlmProviderKind;
  enabled: boolean;
  base_url: string | null;
  api_key_credential_id: string | null;
  default_model: string | null;
  is_local: boolean;
  auto_detected: boolean;
  extra: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface LlmModel {
  id: string;
  name: string;
  provider_id: string;
  context_window: number | null;
  supports_tools: boolean;
  supports_vision: boolean;
  is_local: boolean;
  size_bytes: number | null;
  family: string | null;
}

export interface AgentLlmCompatibility {
  agent_provider: string;
  llm_provider_kind: LlmProviderKind;
  supports_model_override: boolean;
  notes: string | null;
}

export interface CreateLlmProviderRequest {
  name: string;
  kind: LlmProviderKind;
  enabled?: boolean;
  base_url?: string;
  api_key?: string;
  default_model?: string;
  is_local?: boolean;
  extra?: Record<string, unknown>;
}

export interface UpdateLlmProviderRequest {
  name?: string;
  kind?: LlmProviderKind;
  enabled?: boolean;
  base_url?: string | null;
  api_key?: string;
  default_model?: string | null;
  is_local?: boolean;
  extra?: Record<string, unknown>;
}

export interface LlmProviderTestResult {
  success: boolean;
  message: string;
  latency_ms: number | null;
}

export interface DetectedLlmProvider {
  kind: LlmProviderKind;
  name: string;
  base_url: string;
  version: string | null;
  already_registered: boolean;
}

// --- API Error ---

export interface ApiError {
  error: {
    code: string;
    message: string;
  };
}
