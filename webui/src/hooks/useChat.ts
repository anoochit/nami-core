import { useState, useEffect, useCallback } from 'react';
import { api } from '../lib/api';
import { processSlashCommand } from '../lib/commandLoader';
import { shouldAddSpace } from '../lib/utils';
import type { Message, Thread, AgentEvent } from '../types/chat';

export const useChat = () => {
  const [threads, setThreads] = useState<Thread[]>([{ id: '1', title: 'New Conversation', messages: [] }]);
  const [activeThreadId, setActiveThreadId] = useState<string>('1');
  const [input, setInput] = useState('');
  const [promptHistory, setPromptHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [previewPath, setPreviewPath] = useState<string | null>(null);
  const [previewWikiPath, setPreviewWikiPath] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const activeThread = threads.find(t => t.id === activeThreadId) || threads[0];

  useEffect(() => {
    const initActiveThreadSession = async () => {
      if (activeThread && !activeThread.sessionId) {
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeThreadId]); 

  const updateActiveThread = useCallback((updater: (thread: Thread) => Thread) => {
    setThreads(prev => prev.map(t => t.id === activeThreadId ? updater(t) : t));
  }, [activeThreadId]);

  const sendMessage = async () => {
    if (!input.trim() || isLoading) return;

    const originalInput = input;
    const processedInput = processSlashCommand(input);

    setPromptHistory(prev => [...prev, originalInput]);
    setHistoryIndex(-1);
    setInput('');
    setError(null);

    const newUserMessage: Message = { 
        id: Date.now().toString(), 
        sender: 'user', 
        text: originalInput 
    };
    
    updateActiveThread(t => ({ ...t, messages: [...t.messages, newUserMessage] }));

    if (processedInput.startsWith('###')) {
        updateActiveThread(t => ({ 
            ...t, 
            messages: [...t.messages, { 
                id: Date.now().toString() + '_local', 
                sender: 'agent', 
                text: processedInput 
            }] 
        }));
        return;
    }

    let currentSessionId = activeThread.sessionId;

    if (!currentSessionId) {
      try {
        const session = await api.createSession('nami', 'user1');
        currentSessionId = session.session_id;
        updateActiveThread(t => ({ ...t, sessionId: currentSessionId }));
      } catch (e: unknown) {
        const msg = e instanceof Error ? e.message : "Failed to create session";
        setError(msg);
        return;
      }
    }

    const agentMsgId = Date.now().toString() + '_agent';
    updateActiveThread(t => ({ 
        ...t, 
        messages: [...t.messages, { id: agentMsgId, sender: 'agent', text: '' }] 
    }));

    setIsLoading(true);
    
    try {
      await api.runAgent('nami', 'user1', currentSessionId!, {
        role: 'user',
        parts: [{ text: processedInput }]
      }, (data: AgentEvent) => { 
          const parts = data?.content?.parts;
          if (!Array.isArray(parts)) return;

          updateActiveThread(t => {
              const messages = [...t.messages];
              const agentMsgIndex = messages.findIndex(m => m.id === agentMsgId);
              if (agentMsgIndex === -1) return t;

              const msg = { ...messages[agentMsgIndex] };

              parts.forEach((p) => {
                  if (p.text) {
                      let chunk = p.text;
                      if (chunk.startsWith('-') && msg.text.length > 0 && !msg.text.endsWith('\n')) {
                          chunk = '\n' + chunk;
                      }
                      
                      if (shouldAddSpace(msg.text, chunk)) {
                          msg.text += ' ' + chunk;
                      } else {
                          msg.text += chunk;
                      }
                  } else if (p.name) {
                      msg.toolCall = { 
                        name: p.name, 
                        args: p.args, 
                        status: 'pending' 
                      };
                  } else if (p.functionResponse) {
                      if (msg.toolCall) {
                          msg.toolCall.status = 'complete';
                          msg.toolCall.result = p.functionResponse.response;
                      }
                  }
              });

              messages[agentMsgIndex] = msg;
              return { ...t, messages };
          });
      });
    } catch (e: unknown) {
      console.error("Agent execution failed", e);
      const msg = e instanceof Error ? e.message : "An unexpected error occurred";
      setError(msg);
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
      setThreads(prev => [newThread, ...prev]);
      setActiveThreadId(newThread.id);
    } catch (e: unknown) {
      console.error("Failed to create session for new thread", e);
      const msg = e instanceof Error ? e.message : "Failed to initialize new chat session.";
      setError(msg);
    }
  };

  const navigateHistory = (direction: 'up' | 'down') => {
    if (promptHistory.length === 0) return;

    let newIndex: number;
    if (direction === 'up') {
        newIndex = historyIndex === -1 ? promptHistory.length - 1 : Math.max(0, historyIndex - 1);
    } else if (historyIndex !== -1) {
        newIndex = historyIndex + 1;
        if (newIndex >= promptHistory.length) {
            setInput('');
            setHistoryIndex(-1);
            return;
        }
    } else {
        return;
    }

    setHistoryIndex(newIndex);
    setInput(promptHistory[newIndex]);
  };

  const clearMessages = () => {
    updateActiveThread(t => ({ ...t, messages: [] }));
  };

  return {
    threads,
    activeThread,
    activeThreadId,
    input,
    sidebarOpen,
    previewPath,
    previewWikiPath,
    isLoading,
    error,
    setActiveThreadId,
    setInput,
    setSidebarOpen,
    setPreviewPath,
    setPreviewWikiPath,
    sendMessage,
    createNewThread,
    navigateHistory,
    clearMessages
  };
};
