import { useState, useEffect, useCallback, useRef } from 'react';
import { api } from '../lib/api';
import { processSlashCommand } from '../lib/commandLoader';
import { shouldAddSpace } from '../lib/utils';
import type { Message, Thread, AgentEvent, Attachment } from '../types/chat';

interface QueuedMessage {
  text: string;
  attachments: Attachment[];
  threadId: string;
}

export const useChat = () => {
  const [threads, setThreads] = useState<Thread[]>([{ id: '1', title: 'Conversation', messages: [] }]);
  const [activeThreadId, setActiveThreadId] = useState<string>('1');
  const [input, setInput] = useState('');
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  const [messageQueue, setMessageQueue] = useState<QueuedMessage[]>([]);
  const [promptHistory, setPromptHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [previewPath, setPreviewPath] = useState<string | null>(null);
  const [previewWikiPath, setPreviewWikiPath] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const threadsRef = useRef(threads);
  useEffect(() => {
    threadsRef.current = threads;
  }, [threads]);

  const updateThreadById = useCallback((id: string, updater: (thread: Thread) => Thread) => {
    setThreads(prev => prev.map(t => t.id === id ? updater(t) : t));
  }, []);

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
  updateThreadById(activeThreadId, updater);
}, [activeThreadId, updateThreadById]);

  const addAttachments = async (files: FileList | File[]) => {
    const newAttachments: Attachment[] = Array.from(files).map(file => ({
      id: Math.random().toString(36).substring(7),
      name: file.name,
      status: 'uploading'
    }));

    setAttachments(prev => [...prev, ...newAttachments]);

    // Handle uploads
    Array.from(files).forEach(async (file, index) => {
      const attId = newAttachments[index].id;
      try {
        const paths = await api.uploadFile(file);
        setAttachments(prev => prev.map(a => 
          a.id === attId ? { ...a, status: 'success', path: paths[0] } : a
        ));
      } catch (e: unknown) {
        const errorMsg = e instanceof Error ? e.message : String(e);
        setAttachments(prev => prev.map(a => 
          a.id === attId ? { ...a, status: 'error', error: errorMsg } : a
        ));
      }
    });
  };

  const removeAttachment = (id: string) => {
    setAttachments(prev => prev.filter(a => a.id !== id));
  };

  const executeMessage = useCallback(async (queuedMsg: QueuedMessage) => {
    const { text: originalInput, attachments: currentAttachments, threadId } = queuedMsg;
    const processedInput = processSlashCommand(originalInput);

    if (processedInput.startsWith('###')) {
        updateThreadById(threadId, t => ({ 
            ...t, 
            messages: [...t.messages, { 
                id: Date.now().toString() + '_local', 
                sender: 'agent', 
                text: processedInput 
            }] 
        }));
        return;
    }

    const thread = threadsRef.current.find(t => t.id === threadId);
    if (!thread) return;

    let currentSessionId = thread.sessionId;

    if (!currentSessionId) {
      try {
        const session = await api.createSession('nami', 'user1');
        currentSessionId = session.session_id;
        updateThreadById(threadId, t => ({ ...t, sessionId: currentSessionId }));
      } catch (e: unknown) {
        const msg = e instanceof Error ? e.message : "Failed to create session";
        setError(msg);
        return;
      }
    }

    const agentMsgId = Date.now().toString() + '_agent';
    updateThreadById(threadId, t => ({ 
        ...t, 
        messages: [...t.messages, { id: agentMsgId, sender: 'agent', text: '' }] 
    }));

    setIsLoading(true);
    
    let finalPrompt = processedInput;
    if (currentAttachments.length > 0) {
        const attachmentNotes = currentAttachments
            .filter(a => a.status === 'success')
            .map(a => `[File attached: ${a.name} at ${a.path}]`)
            .join('\n');
        if (attachmentNotes) {
            finalPrompt = `${attachmentNotes}\n\n${finalPrompt}`;
        }
    }

    try {
      await api.runAgent('nami', 'user1', currentSessionId!, {
        role: 'user',
        parts: [{ text: finalPrompt }]
      }, (data: AgentEvent) => { 
          const parts = data?.content?.parts;
          if (!Array.isArray(parts)) return;

          updateThreadById(threadId, t => {
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
  }, [updateThreadById]);

  useEffect(() => {
    if (!isLoading && messageQueue.length > 0) {
      const nextMessage = messageQueue[0];
      setMessageQueue(prev => prev.slice(1));
      executeMessage(nextMessage);
    }
  }, [isLoading, messageQueue, executeMessage]);

  const sendMessage = useCallback(async () => {
    if (!input.trim() && attachments.length === 0) return;

    const originalInput = input;
    const currentAttachments = [...attachments];
    const targetThreadId = activeThreadId;

    setPromptHistory(prev => [...prev, originalInput]);
    setHistoryIndex(-1);
    setInput('');
    setAttachments([]);
    setError(null);

    const newUserMessage: Message = { 
        id: Date.now().toString(), 
        sender: 'user', 
        text: originalInput,
        attachments: currentAttachments
    };
    
    updateThreadById(targetThreadId, t => ({ ...t, messages: [...t.messages, newUserMessage] }));

    setMessageQueue(prev => [...prev, {
      text: originalInput,
      attachments: currentAttachments,
      threadId: targetThreadId
    }]);
  }, [input, attachments, activeThreadId, updateThreadById]);

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
        title: 'Conversation', 
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
    clearMessages,
    attachments,
    addAttachments,
    removeAttachment,
    messageQueue
  };
};
