import type { Session, AgentContent, AgentEvent } from '../types/chat';

export let BASE_URL = import.meta.env.VITE_API_BASE_URL || "";

// Detect if running inside Tauri and dynamically set BASE_URL using get_api_port command
const initTauriPort = async () => {
  if (typeof window !== 'undefined' && (window as any).__TAURI__) {
    try {
      const port = await (window as any).__TAURI__.core.invoke('get_api_port');
      BASE_URL = `http://127.0.0.1:${port}`;
      console.log(`[Tauri] Dynamic API BASE_URL resolved to: ${BASE_URL}`);
    } catch (e) {
      console.error('[Tauri] Failed to fetch dynamic API port:', e);
    }
  }
};

initTauriPort();

/**
 * Gets the current API key from localStorage or environment
 */
export const getApiKey = () => {
  return localStorage.getItem('nami_api_key') || (import.meta.env.VITE_NAMI_API_KEY as string) || "";
};

/**
 * Sets the API key in localStorage
 */
export const setApiKey = (key: string) => {
  localStorage.setItem('nami_api_key', key);
};

/**
 * Common headers for all API requests
 */
export const getHeaders = (extraHeaders: Record<string, string> = {}) => {
  const headers: Record<string, string> = {
    ...extraHeaders
  };
  
  const apiKey = getApiKey();
  if (apiKey) {
    headers['X-API-Key'] = apiKey;
  }
  
  return headers;
};

export const api = {
  checkHealth: async (): Promise<boolean> => {
    try {
      const response = await fetch(`${BASE_URL}/api/health`, {
        headers: getHeaders(),
      });
      return response.ok;
    } catch {
      return false;
    }
  },

  createSession: async (appName: string, userId: string): Promise<Session> => {
    const sessionId = crypto.randomUUID();
    const response = await fetch(`${BASE_URL}/api/sessions/create`, {
      method: "POST",
      headers: getHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify({
        appName,
        userId,
        sessionId,
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(
        errorData.message ||
          errorData.error ||
          `Session creation failed (${response.status})`,
      );
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
        headers: getHeaders({ "Content-Type": "application/json" }),
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
      throw new Error(
        errorText || `Agent request failed with status ${response.status}`,
      );
    }

    if (!response.body)
      throw new Error("No response body received from server");

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
            console.error(
              "Failed to parse SSE JSON chunk:",
              e,
              "Line:",
              trimmedLine,
            );
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  },

  readWorkspaceFile: async (path: string): Promise<{ content: string }> => {
    const response = await fetch(`${BASE_URL}/api/workspace/read/${path}`, {
      headers: getHeaders(),
    });
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(
        errorData.message ||
          errorData.error ||
          `Failed to read file (${response.status})`,
      );
    }
    return response.json();
  },

  uploadFile: async (file: File): Promise<string[]> => {
    const formData = new FormData();
    formData.append("file", file);

    const response = await fetch(`${BASE_URL}/api/workspace/upload`, {
      method: "POST",
      headers: getHeaders(), // Note: fetch will handle Content-Type for FormData
      body: formData,
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(
        errorData.message ||
          errorData.error ||
          `Upload failed (${response.status})`,
      );
    }

    const data = await response.json();
    return data.paths;
  },

  listWorkspaceFiles: async (
    path: string = "",
  ): Promise<{ entries: Array<{ name: string; type: string }> }> => {
    // Remove leading/trailing slashes for consistent path handling
    const cleanPath = path.replace(/^\/+|\/+$/g, "");
    const url = cleanPath
      ? `${BASE_URL}/api/workspace/files/${cleanPath}`
      : `${BASE_URL}/api/workspace/files`;

    const response = await fetch(url, {
      headers: getHeaders(),
    });
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(
        errorData.message ||
          errorData.error ||
          `Failed to list files (${response.status})`,
      );
    }
    return response.json();
  },

  listWikiPages: async (): Promise<{ pages: string[] }> => {
    const response = await fetch(`${BASE_URL}/api/wiki/pages`, {
      headers: getHeaders(),
    });
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(
        errorData.message ||
          errorData.error ||
          `Failed to list wiki pages (${response.status})`,
      );
    }
    return response.json();
  },

  listSessions: async (): Promise<Array<{session_id: string, app_name: string, user_id: string, created_at: string}>> => {
    const response = await fetch(`${BASE_URL}/api/sessions/list`, {
        headers: getHeaders()
    });
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || errorData.error || `Failed to list sessions (${response.status})`);
    }
    const data = await response.json();
    return data.sessions;
  },

  getSessionMessages: async (sessionId: string): Promise<Array<{llm_response: string, author: string, timestamp: string}>> => {
      const response = await fetch(`${BASE_URL}/api/sessions/${sessionId}/messages`, {
          headers: getHeaders()
      });
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || errorData.error || `Failed to fetch session messages (${response.status})`);
      }
      const data = await response.json();
      return data.messages;
  },
};
