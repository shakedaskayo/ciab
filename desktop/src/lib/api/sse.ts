import { createEventSource } from "./client";
import type { StreamEvent, StreamEventType } from "./types";

export type StreamEventHandler = (event: StreamEvent) => void;

export interface StreamConnection {
  close: () => void;
  /** The ID of the last event received, used for reconnection replay. */
  lastEventId: string | null;
}

// All event types the backend can send as named SSE events
const SSE_EVENT_TYPES: StreamEventType[] = [
  "connected",
  "reconnect",
  "keepalive",
  "text_delta",
  "text_complete",
  "thinking_delta",
  "tool_use_start",
  "tool_input_delta",
  "tool_use_complete",
  "tool_result",
  "tool_progress",
  "sandbox_state_changed",
  "provisioning_step",
  "provisioning_complete",
  "provisioning_failed",
  "session_created",
  "session_completed",
  "session_failed",
  "permission_request",
  "permission_response",
  "user_input_request",
  "result_error",
  "subagent_start",
  "subagent_end",
  "queue_updated",
  "file_changed",
  "error",
  "stats",
  "log_line",
];

function attachListeners(
  es: EventSource,
  conn: StreamConnection,
  onEvent: StreamEventHandler,
  onError?: (error: Event) => void,
  onOpen?: () => void
): void {
  es.onopen = () => {
    onOpen?.();
  };
  // Handle unnamed events (fallback)
  es.onmessage = (event) => {
    try {
      if (event.lastEventId) conn.lastEventId = event.lastEventId;
      const data = JSON.parse(event.data) as StreamEvent;
      onEvent(data);
    } catch {
      // ignore parse errors
    }
  };

  // Handle named events — the backend sends .event(event_type).data(json)
  for (const eventType of SSE_EVENT_TYPES) {
    es.addEventListener(eventType, ((event: MessageEvent) => {
      try {
        if (event.lastEventId) conn.lastEventId = event.lastEventId;
        const data = JSON.parse(event.data) as StreamEvent;
        onEvent(data);
      } catch {
        // ignore parse errors
      }
    }) as EventListener);
  }

  es.onerror = (event) => {
    onError?.(event);
  };
}

export function subscribeSandboxStream(
  sandboxId: string,
  onEvent: StreamEventHandler,
  onError?: (error: Event) => void,
  lastEventId?: string | null,
  onOpen?: () => void
): StreamConnection {
  const suffix = lastEventId
    ? `?last_event_id=${encodeURIComponent(lastEventId)}`
    : "";
  const es = createEventSource(
    `/api/v1/sandboxes/${sandboxId}/stream${suffix}`
  );
  const conn: StreamConnection = {
    close: () => es.close(),
    lastEventId: lastEventId ?? null,
  };
  attachListeners(es, conn, onEvent, onError, onOpen);
  return conn;
}

export function subscribeSessionStream(
  sessionId: string,
  onEvent: StreamEventHandler,
  onError?: (error: Event) => void,
  lastEventId?: string | null,
  onOpen?: () => void
): StreamConnection {
  const suffix = lastEventId
    ? `?last_event_id=${encodeURIComponent(lastEventId)}`
    : "";
  const es = createEventSource(
    `/api/v1/sessions/${sessionId}/stream${suffix}`
  );
  const conn: StreamConnection = {
    close: () => es.close(),
    lastEventId: lastEventId ?? null,
  };
  attachListeners(es, conn, onEvent, onError, onOpen);
  return conn;
}
