import { useState, useEffect } from 'react';
import { api } from '../lib/api';

export interface Message {
  id: string;
  sender: 'user' | 'agent';
  text: string;
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
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

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

    const newUserMessage: Message = { id: Date.now().toString(), sender: 'user', text: input };
    
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
    await api.runAgent('nami', 'user1', currentSessionId!, {
      role: 'user',
      parts: [{ text: input }]
    }, (data) => {
        const parts = data?.content?.parts;
        const fragment = Array.isArray(parts) ? parts.map((p: any) =>  (p.text || '')).join('') : '';

        setThreads(prev => prev.map(t => 
            t.id === activeThreadId ? { 
                ...t, 
                messages: t.messages.map(m => {
                    if (m.id === agentMsgId) {
                        return { ...m, text: m.text + fragment };
                    }
                    return m;
                })
            } : t
        ));
    });
    setIsLoading(false);
  };

  const createNewThread = async () => {
    const isHealthy = await api.checkHealth();
    if (!isHealthy) {
        alert("Cannot connect to the backend server. Please ensure it is running.");
        return;
    }
    
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
    } catch (e) {
      console.error("Failed to create session for new thread", e);
      alert("Failed to initialize new chat session.");
    }
  };

  return {
    threads,
    activeThread,
    activeThreadId,
    input,
    sidebarOpen,
    isLoading,
    setActiveThreadId,
    setInput,
    setSidebarOpen,
    sendMessage,
    createNewThread
  };
};
