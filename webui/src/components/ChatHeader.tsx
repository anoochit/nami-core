import React from 'react';
import { 
  //  PanelLeftOpen,
  //  PanelLeftClose,
   Trash2,
   FileText,
   RefreshCcw } from 'lucide-react';

interface ChatHeaderProps {
  title: string;
  sessionId?: string;
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
  onClear?: () => void;
  onPreview?: () => void;
  onNewThread?: () => void;
}

export const ChatHeader: React.FC<ChatHeaderProps> = ({ 
  title, 
  sessionId, 
  // sidebarOpen, 
  // onToggleSidebar,
  onClear,
  onPreview,
  onNewThread
}) => {
  return (
    <header className="h-14 border-b border-slate-100 flex items-center px-4 justify-between bg-white/80 backdrop-blur-md sticky top-0 z-10 shadow-[0_1px_3px_rgba(0,0,0,0.01)]">
      {/* <button 
        onClick={onToggleSidebar} 
        className="p-1.5 text-slate-400 hover:text-slate-800 hover:bg-slate-50 rounded-lg transition-all duration-200"
        aria-label={sidebarOpen ? "Close sidebar" : "Open sidebar"}
      >
        {sidebarOpen ? <PanelLeftClose size={18} /> : <PanelLeftOpen size={18} />}
      </button> */}
      
      <div className="flex flex-col items-center max-w-[50%]">
        <h2 className="font-display font-semibold text-sm text-slate-800 truncate max-w-[200px] sm:max-w-md text-center leading-snug">
          {title}
        </h2>
        {sessionId && (
          <div className="mt-0.5 text-[9px] font-mono text-slate-400 uppercase tracking-widest">
            ID: {sessionId.substring(0, 16)}
          </div>
        )}
      </div>

      <div className="flex items-center gap-1">
        {onNewThread && (
          <button 
            onClick={onNewThread} 
            className="p-1.5 text-slate-400 hover:text-slate-800 hover:bg-slate-50 rounded-lg transition-all duration-200"
            title="New Chat"
            aria-label="New chat"
          >
            <RefreshCcw size={18} />
          </button>
        )}
        {onPreview && (
          <button 
            onClick={onPreview} 
            className="p-1.5 text-slate-400 hover:text-blue-600 hover:bg-blue-50/50 rounded-lg transition-all duration-200"
            title="Preview Latest File"
            aria-label="Preview"
          >
            <FileText size={18} />
          </button>
        )}
        {onClear && (
          <button 
            onClick={onClear} 
            className="p-1.5 text-slate-400 hover:text-red-500 hover:bg-red-50/50 rounded-lg transition-all duration-200"
            title="Clear Conversation"
            aria-label="Clear chat"
          >
            <Trash2 size={18} />
          </button>
        )}
      </div>
    </header>
  );
};
