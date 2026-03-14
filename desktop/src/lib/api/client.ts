import { useConnectionStore } from "@/lib/stores/connection-store";
import type { ApiError } from "./types";

export class CiabApiError extends Error {
  constructor(
    public code: string,
    message: string,
    public status: number
  ) {
    super(message);
    this.name = "CiabApiError";
  }
}

function getBaseUrl(): string {
  return useConnectionStore.getState().serverUrl;
}

function getApiKey(): string {
  return useConnectionStore.getState().apiKey;
}

function getHeaders(): Record<string, string> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  const apiKey = getApiKey();
  if (apiKey) {
    headers["Authorization"] = `Bearer ${apiKey}`;
  }
  return headers;
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    let errorBody: ApiError | null = null;
    try {
      const text = await response.text();
      // CIAB returns JSON errors; a plain-text 404 means we're hitting the wrong server
      try {
        errorBody = JSON.parse(text);
      } catch {
        // Not JSON — likely not the CIAB server
        if (response.status === 404 && !text.includes('"error"')) {
          throw new CiabApiError(
            "not_ciab_server",
            `Server at ${getBaseUrl()} returned 404 — is the CIAB server running on this port?`,
            response.status
          );
        }
      }
    } catch (e) {
      if (e instanceof CiabApiError) throw e;
      // ignore other parse failures
    }
    throw new CiabApiError(
      errorBody?.error?.code ?? "unknown",
      errorBody?.error?.message ?? `HTTP ${response.status}`,
      response.status
    );
  }
  if (response.status === 204) {
    return undefined as T;
  }
  return response.json();
}

export async function get<T>(path: string): Promise<T> {
  const response = await fetch(`${getBaseUrl()}${path}`, {
    method: "GET",
    headers: getHeaders(),
  });
  return handleResponse<T>(response);
}

export async function post<T>(path: string, body?: unknown): Promise<T> {
  const response = await fetch(`${getBaseUrl()}${path}`, {
    method: "POST",
    headers: getHeaders(),
    body: body ? JSON.stringify(body) : undefined,
  });
  return handleResponse<T>(response);
}

export async function put<T>(path: string, body?: unknown): Promise<T> {
  const response = await fetch(`${getBaseUrl()}${path}`, {
    method: "PUT",
    headers: getHeaders(),
    body: body ? JSON.stringify(body) : undefined,
  });
  return handleResponse<T>(response);
}

export async function del<T>(path: string): Promise<T> {
  const response = await fetch(`${getBaseUrl()}${path}`, {
    method: "DELETE",
    headers: getHeaders(),
  });
  return handleResponse<T>(response);
}

export async function putRaw(path: string, data: ArrayBuffer): Promise<void> {
  const headers = getHeaders();
  headers["Content-Type"] = "application/octet-stream";
  const response = await fetch(`${getBaseUrl()}${path}`, {
    method: "PUT",
    headers,
    body: data,
  });
  await handleResponse<void>(response);
}

export async function getRaw(path: string): Promise<ArrayBuffer> {
  const response = await fetch(`${getBaseUrl()}${path}`, {
    method: "GET",
    headers: getHeaders(),
  });
  if (!response.ok) {
    throw new CiabApiError("download_failed", `HTTP ${response.status}`, response.status);
  }
  return response.arrayBuffer();
}

/**
 * Create an EventSource for SSE streaming.
 *
 * Browser EventSource cannot set custom headers, so the API key is passed
 * via the `?token=` query parameter. The backend's auth middleware accepts
 * this as an alternative to the Authorization header.
 */
export function createEventSource(path: string): EventSource {
  const base = getBaseUrl();
  const apiKey = getApiKey();
  const separator = path.includes("?") ? "&" : "?";
  const url = apiKey
    ? `${base}${path}${separator}token=${encodeURIComponent(apiKey)}`
    : `${base}${path}`;
  return new EventSource(url);
}
