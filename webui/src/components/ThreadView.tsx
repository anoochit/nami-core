import React from 'react';
import { Bot, Send, ChevronLeft, ChevronRight } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import { cn } from '../lib/utils';

interface Message {
  id: string;
  sender: 'user' | 'agent';
  text: string;
}

interface Thread {
  id: string;
  title: string;
  messages: Message[];
  sessionId?: string;
}

interface ThreadViewProps {
  thread: Thread;
  input: string;
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
  onInputChange: (val: string) => void;
  onSendMessage: () => void;
}

export const ThreadView: React.FC<ThreadViewProps> = ({ thread, input, sidebarOpen, onToggleSidebar, onInputChange, onSendMessage }) => {
  return (
    <div className="flex-1 flex flex-col h-full">
      <header className="h-14 border-b flex items-center px-4 justify-between">
        <button onClick={onToggleSidebar} className="p-2 hover:bg-gray-100 rounded-md">
          {sidebarOpen ? <ChevronLeft size={20} /> : <ChevronRight size={20} />}
        </button>
        <h2 className="font-semibold">
          {thread.title} {thread.sessionId && <span className="text-xs text-gray-500 font-normal">({thread.sessionId})</span>}
        </h2>
        <div className="w-8"></div>
      </header>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {thread.messages.map(m => (
          <div key={m.id} className={cn("flex gap-3", m.sender === 'user' ? "justify-end" : "justify-start")}>
            {m.sender === 'agent' && <div className="w-8 h-8 flex items-center justify-center bg-gray-800 text-white rounded-full shrink-0"><Bot size={16} /></div>}
            <div className={cn("p-3 rounded-2xl max-w-[70%] prose prose-sm", m.sender === 'user' ? "bg-black text-white" : "bg-gray-100 text-gray-800")}>
              {m.sender === 'agent' ? (
                <ReactMarkdown>{m.text}</ReactMarkdown>
              ) : (
                m.text
              )}
            </div>
          </div>
        ))}
      </div>

      <div className="p-4 border-t">
        <div className="flex gap-2 max-w-3xl mx-auto border rounded-full p-1 shadow-sm focus-within:ring-2 focus-within:ring-black">
          <input 
            value={input} 
            onChange={(e) => onInputChange(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && onSendMessage()}
            className="flex-1 bg-transparent px-4 py-2 outline-none" 
            placeholder="Message..."
          />
          <button onClick={onSendMessage} className="bg-black text-white p-3 rounded-full hover:bg-gray-800 transition-colors"><Send size={18} /></button>
        </div>
      </div>
    </div>
  );
};
