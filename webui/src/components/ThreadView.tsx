import React, { useState } from 'react';
import type { Thread, Attachment } from '../types/chat';
import { ChatHeader } from './ChatHeader';
import { MessageList } from './MessageList';
import { ChatInput } from './ChatInput';
import { CirclePlus } from 'lucide-react';

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
  onClear: () => void;
  onNewThread: () => void;
  isLoading: boolean;
  error?: string | null;
  attachments: Attachment[];
  onAddAttachments: (files: FileList | File[]) => void;
  onRemoveAttachment: (id: string) => void;
  queueCount?: number;
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
  onClear,
  onNewThread,
  isLoading, 
  error,
  attachments,
  onAddAttachments,
  onRemoveAttachment,
  queueCount = 0
}) => {
  const [isDragging, setIsDragging] = useState(false);

  const handlePreview = () => {
    const lastMessageWithFile = [...thread.messages]
      .reverse()
      .find(m => m.toolCall?.status === 'complete' && 
                 ((m.toolCall.result as any)?.path || (m.toolCall.result as any)?.filename));

    if (lastMessageWithFile) {
        const path = (lastMessageWithFile.toolCall!.result as any).path || (lastMessageWithFile.toolCall!.result as any).filename;
        onPreviewFile?.(path);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    if (e.currentTarget.contains(e.relatedTarget as Node)) return;
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      onAddAttachments(e.dataTransfer.files);
    }
  };

  return (
    <div 
      className="flex-1 flex flex-col h-full bg-white relative overflow-hidden"
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {isDragging && (
        <div className="absolute inset-0 z-50 bg-black/5 backdrop-blur-[2px] flex items-center justify-center pointer-events-none">
          <div className="bg-white p-8 rounded-3xl shadow-2xl flex flex-col items-center gap-4 border-2 border-dashed border-gray-300 scale-110 transition-transform duration-300">
             <div className="w-16 h-16 bg-black text-white rounded-full flex items-center justify-center shadow-lg">
                <CirclePlus size={32} />
             </div>
             <p className="font-bold text-xl">Drop files here</p>
          </div>
        </div>
      )}

      <ChatHeader 
        title={thread.title} 
        sessionId={thread.sessionId} 
        sidebarOpen={sidebarOpen} 
        onToggleSidebar={onToggleSidebar} 
        onClear={onClear}
        onNewThread={onNewThread}
        onPreview={handlePreview}
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
        attachments={attachments}
        onAddAttachments={onAddAttachments}
        onRemoveAttachment={onRemoveAttachment}
        queueCount={queueCount}
      />
    </div>
  );
};
