import { useEffect, useRef, useState, useCallback } from "react";
import {
  subscribeSandboxStream,
  subscribeSessionStream,
  type StreamConnection,
} from "@/lib/api/sse";
import type { StreamEvent } from "@/lib/api/types";

/** Maximum number of events to keep in the sliding window. */
const MAX_EVENTS = 500;

/** Reconnect delay in milliseconds. Uses exponential backoff. */
const BASE_RECONNECT_MS = 1000;
const MAX_RECONNECT_MS = 15000;

export function useSandboxStream(
  sandboxId: string | null,
  onEvent?: (event: StreamEvent) => void
) {
  const [events, setEvents] = useState<StreamEvent[]>([]);
  const [connected, setConnected] = useState(false);
  const connRef = useRef<StreamConnection | null>(null);
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;
  const seenIdsRef = useRef(new Set<string>());

  useEffect(() => {
    if (!sandboxId) return;

    let closed = false;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let attempt = 0;

    function connect() {
      if (closed) return;

      connRef.current?.close();
      const lastId = connRef.current?.lastEventId ?? null;

      const conn = subscribeSandboxStream(
        sandboxId!,
        (event) => {
          // Deduplicate replayed events
          if (seenIdsRef.current.has(event.id)) return;
          seenIdsRef.current.add(event.id);
          // Keep set from growing unbounded
          if (seenIdsRef.current.size > MAX_EVENTS * 2) {
            const arr = [...seenIdsRef.current];
            seenIdsRef.current = new Set(arr.slice(-MAX_EVENTS));
          }

          setConnected(true);
          attempt = 0;
          setEvents((prev) => [...prev.slice(-(MAX_EVENTS - 1)), event]);
          onEventRef.current?.(event);
        },
        () => {
          setConnected(false);
          if (!closed) {
            const delay = Math.min(
              BASE_RECONNECT_MS * Math.pow(2, attempt),
              MAX_RECONNECT_MS
            );
            attempt++;
            reconnectTimer = setTimeout(connect, delay);
          }
        },
        lastId,
        () => {
          setConnected(true);
          attempt = 0;
        }
      );
      connRef.current = conn;
    }

    connect();

    return () => {
      closed = true;
      if (reconnectTimer) clearTimeout(reconnectTimer);
      connRef.current?.close();
    };
  }, [sandboxId]);

  const clear = useCallback(() => {
    setEvents([]);
    seenIdsRef.current.clear();
  }, []);

  return { events, connected, clear };
}

export function useSessionStream(
  sessionId: string | null,
  onEvent?: (event: StreamEvent) => void
) {
  const [events, setEvents] = useState<StreamEvent[]>([]);
  const [connected, setConnected] = useState(false);
  const connRef = useRef<StreamConnection | null>(null);
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;
  const seenIdsRef = useRef(new Set<string>());

  useEffect(() => {
    if (!sessionId) return;

    let closed = false;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let attempt = 0;

    function connect() {
      if (closed) return;

      connRef.current?.close();
      const lastId = connRef.current?.lastEventId ?? null;

      const conn = subscribeSessionStream(
        sessionId!,
        (event) => {
          // Deduplicate replayed events
          if (seenIdsRef.current.has(event.id)) return;
          seenIdsRef.current.add(event.id);
          if (seenIdsRef.current.size > MAX_EVENTS * 2) {
            const arr = [...seenIdsRef.current];
            seenIdsRef.current = new Set(arr.slice(-MAX_EVENTS));
          }

          setConnected(true);
          attempt = 0;
          setEvents((prev) => [...prev.slice(-(MAX_EVENTS - 1)), event]);
          onEventRef.current?.(event);
        },
        () => {
          setConnected(false);
          if (!closed) {
            const delay = Math.min(
              BASE_RECONNECT_MS * Math.pow(2, attempt),
              MAX_RECONNECT_MS
            );
            attempt++;
            reconnectTimer = setTimeout(connect, delay);
          }
        },
        lastId,
        () => {
          setConnected(true);
          attempt = 0;
        }
      );
      connRef.current = conn;
    }

    connect();

    return () => {
      closed = true;
      if (reconnectTimer) clearTimeout(reconnectTimer);
      connRef.current?.close();
    };
  }, [sessionId]);

  const clear = useCallback(() => {
    setEvents([]);
    seenIdsRef.current.clear();
  }, []);

  return { events, connected, clear };
}
