import React from 'react';
import { PanelLeftOpen, PanelLeftClose, Trash2, Inbox } from 'lucide-react';

interface ChatHeaderProps {
  title: string;
  sessionId?: string;
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
  onClear?: () => void;
  onPreview?: () => void;
}

export const ChatHeader: React.FC<ChatHeaderProps> = ({ 
  title, 
  sessionId, 
  sidebarOpen, 
  onToggleSidebar,
  onClear,
  onPreview
}) => {
  return (
    <header className="h-14 border-b flex items-center px-4 justify-between bg-white sticky top-0 z-10">
      <button 
        onClick={onToggleSidebar} 
        className="p-2 hover:bg-gray-100 rounded-md transition-colors"
        aria-label={sidebarOpen ? "Close sidebar" : "Open sidebar"}
      >
        {sidebarOpen ? <PanelLeftClose size={20} /> : <PanelLeftOpen size={20} />}
      </button>
      <div className="flex flex-col items-center">
        <h2 className="font-semibold text-sm sm:text-base truncate max-w-[200px] sm:max-w-md text-center">
          {title}
        </h2>
        <div className="flex items-center gap-2">
            {sessionId && (
            <div className="text-[10px] text-gray-400 font-normal uppercase tracking-wider">
                ID: {sessionId.slice(0, 8)}...
            </div>
            )}
           
        </div>
      </div>
      <div className="flex items-center gap-1">
        {onClear && (
          <button 
            onClick={onClear} 
            className="p-2 hover:bg-gray-100 rounded-md transition-colors text-gray-600 hover:text-red-600"
            aria-label="Clear chat"
          >
            <Trash2 size={20} />
          </button>
        )}
        {onPreview && (
          <button 
            onClick={onPreview} 
            className="p-2 hover:bg-gray-100 rounded-md transition-colors text-gray-600 hover:text-blue-600"
            aria-label="Preview"
          >
            <Inbox size={20} />
          </button>
        )}
      </div>
    </header>
  );
};
