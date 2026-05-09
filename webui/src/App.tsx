import React, { useState, useEffect } from 'react';
import { Send, Bot, User, MessageSquare, Plus, Trash2, ChevronLeft, ChevronRight } from 'lucide-react';
import { cn } from './lib/utils';

interface Message {
  id: string;
  sender: 'user' | 'agent';
  text: string;
}

interface Thread {
  id: string;
  title: string;
  messages: Message[];
}

export default function App() {
  const [threads, setThreads] = useState<Thread[]>([{ id: '1', title: 'New Conversation', messages: [] }]);
  const [activeThreadId, setActiveThreadId] = useState<string>('1');
  const [input, setInput] = useState('');
  const [sidebarOpen, setSidebarOpen] = useState(true);

  const activeThread = threads.find(t => t.id === activeThreadId) || threads[0];

  const sendMessage = async () => {
    if (!input.trim()) return;

    const newUserMessage: Message = { id: Date.now().toString(), sender: 'user', text: input };
    
    setThreads(prev => prev.map(t => 
      t.id === activeThreadId ? { ...t, messages: [...t.messages, newUserMessage] } : t
    ));
    setInput('');

    // Simulate API call
    setTimeout(() => {
      const agentResponse: Message = { id: Date.now().toString(), sender: 'agent', text: `Echo: ${input}` };
      setThreads(prev => prev.map(t => 
        t.id === activeThreadId ? { ...t, messages: [...t.messages, agentResponse] } : t
      ));
    }, 500);
  };

  const createNewThread = () => {
    const newThread: Thread = { id: Date.now().toString(), title: 'New Conversation', messages: [] };
    setThreads([newThread, ...threads]);
    setActiveThreadId(newThread.id);
  };

  return (
    <div className="flex h-screen bg-white">
      {/* Sidebar */}
      <div className={cn("w-64 border-r bg-gray-50 flex flex-col transition-all duration-300", !sidebarOpen && "-ml-64")}>
        <div className="p-4 border-b">
          <button onClick={createNewThread} className="w-full flex items-center justify-center gap-2 bg-black text-white p-2 rounded-md hover:bg-gray-800">
            <Plus size={18} /> New Chat
          </button>
        </div>
        <div className="flex-1 overflow-y-auto p-2">
          {threads.map(thread => (
            <div 
              key={thread.id} 
              onClick={() => setActiveThreadId(thread.id)}
              className={cn("p-3 rounded-md cursor-pointer flex items-center gap-2 hover:bg-gray-200", activeThreadId === thread.id && "bg-gray-200")}
            >
              <MessageSquare size={16} />
              <span className="truncate">{thread.title}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col">
        <header className="h-14 border-b flex items-center px-4 justify-between">
          <button onClick={() => setSidebarOpen(!sidebarOpen)} className="p-2 hover:bg-gray-100 rounded-md">
            {sidebarOpen ? <ChevronLeft size={20} /> : <ChevronRight size={20} />}
          </button>
          <h2 className="font-semibold">{activeThread.title}</h2>
          <div className="w-8"></div>
        </header>

        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {activeThread.messages.map(m => (
            <div key={m.id} className={cn("flex gap-3", m.sender === 'user' ? "justify-end" : "justify-start")}>
              {m.sender === 'agent' && <div className="w-8 h-8 flex items-center justify-center bg-gray-800 text-white rounded-full"><Bot size={16} /></div>}
              <div className={cn("p-3 rounded-2xl max-w-[70%]", m.sender === 'user' ? "bg-black text-white" : "bg-gray-100")}>
                {m.text}
              </div>
            </div>
          ))}
        </div>

        <div className="p-4 border-t">
          <div className="flex gap-2 max-w-3xl mx-auto border rounded-full p-1 shadow-sm focus-within:ring-2 focus-within:ring-black">
            <input 
              value={input} 
              onChange={(e) => setInput(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
              className="flex-1 bg-transparent px-4 py-2 outline-none" 
              placeholder="Message..."
            />
            <button onClick={sendMessage} className="bg-black text-white p-3 rounded-full hover:bg-gray-800 transition-colors"><Send size={18} /></button>
          </div>
        </div>
      </div>
    </div>
  );
}
