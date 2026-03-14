import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { sessions } from "@/lib/api/endpoints";

export function useSessions(sandboxId: string) {
  return useQuery({
    queryKey: ["sessions", sandboxId],
    queryFn: () => sessions.list(sandboxId),
    enabled: !!sandboxId,
  });
}

export function useSession(sessionId: string) {
  return useQuery({
    queryKey: ["session", sessionId],
    queryFn: () => sessions.get(sessionId),
    enabled: !!sessionId,
    // Don't refetch while agent is processing — ChatView manages this
    refetchInterval: false,
  });
}

export function useCreateSession(sandboxId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (metadata?: Record<string, unknown>) =>
      sessions.create(sandboxId, metadata),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sessions", sandboxId] });
    },
  });
}

export interface SendMessagePayload {
  text: string;
  images?: Array<{ data: string; media_type: string }>;
}

export function useSendMessage(sessionId: string) {
  return useMutation({
    mutationFn: (payload: string | SendMessagePayload) => {
      const text = typeof payload === "string" ? payload : payload.text;
      const images = typeof payload === "string" ? undefined : payload.images;

      const content: Array<{ type: string; text?: string; media_type?: string; data?: string }> = [];

      // Add images first (if any)
      if (images?.length) {
        for (const img of images) {
          content.push({ type: "image", media_type: img.media_type, data: img.data });
        }
      }

      // Add text
      if (text) {
        content.push({ type: "text", text });
      }

      return sessions.sendMessage(sessionId, {
        role: "user",
        content: content as Array<{ type: string; text: string }>,
      });
    },
    // Don't auto-invalidate — ChatView.handleSend manages refetch on success
  });
}

export function useInterruptSession(sessionId: string) {
  return useMutation({
    mutationFn: () => sessions.interrupt(sessionId),
  });
}

export function useSetPermissionMode(sessionId: string) {
  return useMutation({
    mutationFn: (body: {
      mode: string;
      always_require_approval?: string[];
      always_allow?: string[];
    }) => sessions.setPermissionMode(sessionId, body),
  });
}

export function useRespondToPermission(sessionId: string) {
  return useMutation({
    mutationFn: (params: { requestId: string; approved: boolean }) =>
      sessions.respondToPermission(sessionId, params.requestId, params.approved),
  });
}

export function useRespondToUserInput(sessionId: string) {
  return useMutation({
    mutationFn: (params: { requestId: string; answer: string }) =>
      sessions.respondToUserInput(sessionId, params.requestId, params.answer),
  });
}

export interface SessionSkill {
  source: string;
  skill_id?: string;
  name?: string;
}

export function useUpdateSessionSkills(sessionId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (activeSkills: SessionSkill[]) =>
      sessions.updateSkills(sessionId, activeSkills),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["session", sessionId] });
    },
  });
}
