import { AssistantRuntimeProvider, useLocalRuntime } from "@assistant-ui/react";
import { AssistantModal } from "@assistant-ui/react";
import "@assistant-ui/react/styles/index.css";
import { useEffect, useState } from "react";

// Adapter to connect to Nami ADK server via SSE
function useNamiRuntime() {
  const runtime = useLocalRuntime(async (input) => {
    // API endpoint for Nami ADK-Server (REST + SSE)
    const response = await fetch("/api/run_sse", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ prompt: input.text }),
    });

    if (!response.body) throw new Error("No response body");
    
    // Process SSE stream
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const chunk = decoder.decode(value);
      // Logic to handle chunked stream from Nami ADK
      console.log("Chunk:", chunk);
    }
  });

  return runtime;
}

export default function App() {
  const runtime = useNamiRuntime();

  return (
    <AssistantRuntimeProvider runtime={runtime}>
      <div style={{ height: "100vh" }}>
        <h1>Nami Chat</h1>
        <AssistantModal />
      </div>
    </AssistantRuntimeProvider>
  );
}
