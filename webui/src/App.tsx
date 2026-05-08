import { AssistantRuntimeProvider, useLocalRuntime, Thread } from "@assistant-ui/react";
import { AssistantModal } from "@assistant-ui/react";
import "@assistant-ui/react/styles/index.css";
import { useEffect, useState } from "react";
import axios from "axios";

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
    
    // Process SSE stream (simplified for this implementation)
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
  const [threads, setThreads] = useState<any[]>([]);

  // Fetch thread list
  useEffect(() => {
    axios.get(`${API_BASE}/api/list-sessions`)
      .then(res => setThreads(res.data.sessions || []))
      .catch(console.error);
  }, []);

  return (
    <AssistantRuntimeProvider runtime={runtime}>
      <div style={{ display: "flex", height: "100vh" }}>
        {/* Sidebar Thread List */}
        <div style={{ width: "250px", borderRight: "1px solid #ccc", padding: "10px" }}>
          <h2>Threads</h2>
          {threads.map((thread) => (
            <div 
              key={thread.id} 
              onClick={() => setSessionId(thread.id)}
              style={{ cursor: "pointer", padding: "5px", background: sessionId === thread.id ? "#eee" : "transparent" }}
            >
              {thread.id}
            </div>
          ))}
        </div>

        {/* Main Chat Thread */}
        <div style={{ flex: 1 }}>
          {sessionId ? (
            <Thread />
          ) : (
            <div>Please select a thread or start a new one.</div>
          )}
        </div>
      </div>
    </AssistantRuntimeProvider>
  );
}
