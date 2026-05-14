import React from 'react';
import type { Thread } from '../types/chat';
import { ChatHeader } from './ChatHeader';
import { MessageList } from './MessageList';
import { ChatInput } from './ChatInput';

interface ThreadViewProps {
  thread: Thread;
  input: string;
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
  onInputChange: (val: string) => void;
  onSendMessage: () => void;
  onNavigateHistory: (direction: 'up' | 'down') => void;
  onPreviewFile?: (path: string) => void;
  onPreviewWiki?: (title: string) => void;
  isLoading: boolean;
  error?: string | null;
}

export const ThreadView: React.FC<ThreadViewProps> = ({ 
  thread, 
  input, 
  sidebarOpen, 
  onToggleSidebar, 
  onInputChange, 
  onSendMessage, 
  onNavigateHistory,
  onPreviewFile,
  onPreviewWiki,
  isLoading, 
  error 
}) => {
  return (
    <div className="flex-1 flex flex-col h-full bg-white relative overflow-hidden">
      <ChatHeader 
        title={thread.title} 
        sessionId={thread.sessionId} 
        sidebarOpen={sidebarOpen} 
        onToggleSidebar={onToggleSidebar} 
      />

      <MessageList 
        messages={thread.messages} 
        isLoading={isLoading} 
        error={error ?? undefined}
        onPreviewFile={onPreviewFile}
        onPreviewWiki={onPreviewWiki}
      />

      <ChatInput 
        value={input} 
        onInputChange={onInputChange} 
        onSendMessage={onSendMessage} 
        onNavigateHistory={onNavigateHistory} 
        isLoading={isLoading} 
      />
    </div>
  );
};
