import React from 'react';
import { PanelLeftOpen, PanelLeftClose } from 'lucide-react';

interface ChatHeaderProps {
  title: string;
  sessionId?: string;
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
}

export const ChatHeader: React.FC<ChatHeaderProps> = ({ 
  title, 
  sessionId, 
  sidebarOpen, 
  onToggleSidebar 
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
        {sessionId && (
          <div className="text-[10px] text-gray-400 font-normal uppercase tracking-wider">
            ID: {sessionId.slice(0, 8)}...
          </div>
        )}
      </div>
      <div className="w-8"/>
    </header>
  );
};
