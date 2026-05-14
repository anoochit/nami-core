import { useState, useEffect } from 'react';
import { api } from '../lib/api';
import { processSlashCommand } from '../lib/commandLoader';

export interface Message {
  id: string;
  sender: 'user' | 'agent';
  text: string;
  toolCall?: { 
    name: string; 
    args?: any; 
    status: 'pending' | 'complete'; 
    result?: any 
  };
}

export interface Thread {
  id: string;
  title: string;
  messages: Message[];
  sessionId?: string;
}

export const useChat = () => {
  const [threads, setThreads] = useState<Thread[]>([{ id: '1', title: 'New Conversation', messages: [] }]);
  const [activeThreadId, setActiveThreadId] = useState<string>('1');
  const [input, setInput] = useState('');
  const [promptHistory, setPromptHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initActiveThreadSession = async () => {
      const currentThread = threads.find(t => t.id === activeThreadId);
      if (currentThread && !currentThread.sessionId) {
        try {
          const session = await api.createSession('nami', 'user1');
          setThreads(prev => prev.map(t => 
            t.id === activeThreadId ? { ...t, sessionId: session.session_id } : t
          ));
        } catch (e) {
          console.error("Failed to initialize session on load", e);
        }
      }
    };
    initActiveThreadSession();
  }, [activeThreadId]);

  const activeThread = threads.find(t => t.id === activeThreadId) || threads[0];

  const sendMessage = async () => {
    if (!input.trim()) return;

    const processedInput = processSlashCommand(input);

    // Add to history
    setPromptHistory(prev => [...prev, processedInput]);
    setHistoryIndex(-1);

    const newUserMessage: Message = { id: Date.now().toString(), sender: 'user', text: processedInput };
    
    let currentSessionId = activeThread.sessionId;

    if (!currentSessionId) {
      try {
        const session = await api.createSession('nami', 'user1');
        currentSessionId = session.session_id;
        setThreads(prev => prev.map(t => 
          t.id === activeThreadId ? { ...t, sessionId: currentSessionId } : t
        ));
      } catch (e) {
        console.error("Failed to create session", e);
        return;
      }
    }

    setThreads(prev => prev.map(t => 
      t.id === activeThreadId ? { ...t, messages: [...t.messages, newUserMessage] } : t
    ));
    setInput('');

    const agentMsgId = Date.now().toString() + '_agent';
    setThreads(prev => prev.map(t => 
        t.id === activeThreadId ? { ...t, messages: [...t.messages, { id: agentMsgId, sender: 'agent', text: '' }] } : t
    ));

    setIsLoading(true);
    setError(null);
    try {
      await api.runAgent('nami', 'user1', currentSessionId!, {
        role: 'user',
        parts: [{ text: processedInput }]
      }, (data) => {
          const parts = data?.content?.parts;
          if (!Array.isArray(parts)) return;

          setThreads(prev => prev.map(t => {
              if (t.id !== activeThreadId) return t;
              
              const messages = [...t.messages];
              const agentMsgIndex = messages.findIndex(m => m.id === agentMsgId);
              if (agentMsgIndex === -1) return t;

              const msg = { ...messages[agentMsgIndex] };

              const shouldAddSpace = (prev: string, next: string) => {

                  if (!prev || !next) return false;
                  const prevChar = prev.slice(-1);
                  const nextChar = next[0];
                  const isThai = (char: string) => /[\u0E00-\u0E7F]/.test(char);
                  // Add space if switching between Thai and non-Thai, or two separate words
                  return (isThai(prevChar) !== isThai(nextChar));
              };

              parts.forEach((p: any) => {
                  // 1. Text streaming
                  if (p.text) {
                      let formattedText = p.text;
                      // Pre-processor: ensure list markers have preceding newline if not at start
                      if (formattedText.startsWith('-') && msg.text.length > 0 && !msg.text.endsWith('\n')) {
                          formattedText = '\n' + formattedText;
                      }
                      
                      if (shouldAddSpace(msg.text, formattedText)) {
                          msg.text += ' ' + formattedText;
                      } else {
                          msg.text += formattedText;
                      }
                  } 
                  // 2. Tool invocation (part contains name/args)
                  else if (p.name) {
                      msg.toolCall = { name: p.name, args: p.args, status: 'pending' };
                  }
                  // 3. Function response (part contains functionResponse)
                  else if (p.functionResponse) {
                      if (msg.toolCall) {
                          msg.toolCall.status = 'complete';
                          msg.toolCall.result = p.functionResponse.response;
                      }
                  }
              });

              messages[agentMsgIndex] = msg;
              return { ...t, messages };
          }));
      });
    } catch (e: any) {
      console.error("Agent execution failed", e);
      setError(e.message || "An unexpected error occurred");
    } finally {
      setIsLoading(false);
    }
  };


  const createNewThread = async () => {
    const isHealthy = await api.checkHealth();
    if (!isHealthy) {
        setError("Cannot connect to the backend server. Please ensure it is running.");
        return;
    }
    
    setError(null);
    try {
      const session = await api.createSession('nami', 'user1');
      const newThread: Thread = { 
        id: Date.now().toString(), 
        title: 'New Conversation', 
        messages: [],
        sessionId: session.session_id 
      };
      setThreads([newThread, ...threads]);
      setActiveThreadId(newThread.id);
    } catch (e: any) {
      console.error("Failed to create session for new thread", e);
      setError(e.message || "Failed to initialize new chat session.");
    }
  };

  const navigateHistory = (direction: 'up' | 'down') => {
    if (promptHistory.length === 0) return;

    let newIndex = historyIndex;
    if (direction === 'up') {
        newIndex = historyIndex === -1 ? promptHistory.length - 1 : Math.max(0, historyIndex - 1);
    } else if (historyIndex !== -1) {
        newIndex = Math.min(promptHistory.length - 1, historyIndex + 1);
        if (newIndex === promptHistory.length - 1) { // reached latest, clear input
            setInput('');
            setHistoryIndex(-1);
            return;
        }
    } else {
        return; // already at latest
    }

    if (newIndex !== historyIndex) {
        setHistoryIndex(newIndex);
        setInput(promptHistory[newIndex]);
    }
  };

  return {
    threads,
    activeThread,
    activeThreadId,
    input,
    sidebarOpen,
    isLoading,
    error,
    setActiveThreadId,
    setInput,
    setSidebarOpen,
    sendMessage,
    createNewThread,
    navigateHistory
  };
};
