const BASE_URL = "";

export interface Session {
  session_id: string;
}

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
      throw new Error(errorData.message || errorData.error || "Failed to create session");
    }
    return { session_id: sessionId };
  },

  // SSE handler for streaming responses
  runAgent: async (
    appName: string,
    userId: string,
    sessionId: string,
    message: any,
    onMessage: (data: any) => void,
  ) => {
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
      throw new Error(errorText || `Request failed with status ${response.status}`);
    }

    if (!response.body) throw new Error("No response body");

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");

        // Keep the last partial line in the buffer
        buffer = lines.pop() || "";

        for (const line of lines) {
          if (line.trim().startsWith("data: ")) {
            try {
              const event = JSON.parse(line.trim().slice(6));
              console.log("Event:", event);
              onMessage(event);
            } catch (e) {
              console.error("Failed to parse SSE JSON:", e);
            }
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  },
};
