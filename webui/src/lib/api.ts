import type { Session, AgentContent, AgentEvent } from '../types/chat';

const BASE_URL = import.meta.env.VITE_API_BASE_URL || "";

export const api = {
  checkHealth: async (): Promise<boolean> => {
    try {
      const response = await fetch(`${BASE_URL}/api/health`);
      return response.ok;
    } catch {
      return false;
    }
  },

  createSession: async (appName: string, userId: string): Promise<Session> => {
    const sessionId = crypto.randomUUID();
    const response = await fetch(`${BASE_URL}/api/sessions`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        appName,
        userId,
        sessionId,
      }),
    });
    
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || errorData.error || `Session creation failed (${response.status})`);
    }
    
    return { session_id: sessionId };
  },

  /**
   * SSE handler for streaming agent execution responses
   */
  runAgent: async (
    appName: string,
    userId: string,
    sessionId: string,
    message: AgentContent,
    onMessage: (data: AgentEvent) => void,
  ): Promise<void> => {
    const response = await fetch(
      `${BASE_URL}/api/run/${appName}/${userId}/${sessionId}`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          appName,
          userId,
          sessionId,
          new_message: JSON.stringify(message),
          streaming: true,
        }),
      },
    );

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(errorText || `Agent request failed with status ${response.status}`);
    }

    if (!response.body) throw new Error("No response body received from server");

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");

        // Keep the last potentially incomplete line in the buffer
        buffer = lines.pop() || "";

        for (const line of lines) {
          const trimmedLine = line.trim();
          if (!trimmedLine || !trimmedLine.startsWith("data: ")) continue;

          try {
            const jsonStr = trimmedLine.slice(6);
            if (jsonStr === "[DONE]") continue;
            
            const event = JSON.parse(jsonStr);
            onMessage(event);
          } catch (e) {
            console.error("Failed to parse SSE JSON chunk:", e, "Line:", trimmedLine);
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  },

  readWorkspaceFile: async (path: string): Promise<{ content: string }> => {
    const response = await fetch(`${BASE_URL}/api/workspace/read/${path}`);
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || errorData.error || `Failed to read file (${response.status})`);
    }
    return response.json();
  },
};
