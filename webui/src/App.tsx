import { AssistantRuntimeProvider, useLocalRuntime, Thread, ThreadList, AssistantSidebar } from "@assistant-ui/react";
import "@assistant-ui/react/styles/index.css";
import { useState } from "react";

// API Base URL
const API_BASE = "http://localhost:8080";

function useNamiRuntime() {
  const [sessionId, setSessionId] = useState<string | null>(null);

  const runtime = useLocalRuntime(async (input) => {
    if (!sessionId) throw new Error("No active thread");

    const response = await fetch(`${API_BASE}/api/run_sse`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ prompt: input.text, session_id: sessionId }),
    });

    if (!response.body) throw new Error("No response body");
    
    // Process SSE stream
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      console.log("Chunk:", decoder.decode(value));
    }
  });

  return { runtime, sessionId, setSessionId };
}

export default function App() {
  const { runtime, sessionId, setSessionId } = useNamiRuntime();

  return (
    <AssistantRuntimeProvider runtime={runtime}>
      <AssistantSidebar>
        <ThreadList onSelect={(id) => setSessionId(id)} />
      </AssistantSidebar>
      <div style={{ flex: 1, height: "100vh" }}>
        {sessionId ? (
          <Thread />
        ) : (
          <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%" }}>
            Please select a thread or start a new one.
          </div>
        )}
      </div>
    </AssistantRuntimeProvider>
  );
}
