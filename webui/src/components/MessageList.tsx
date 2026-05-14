import React, { useEffect, useRef } from 'react';
import { MessageItem } from './MessageItem';
import type { Message } from '../types/chat';

interface MessageListProps {
  messages: Message[];
  isLoading: boolean;
  error?: string;
}

export const MessageList: React.FC<MessageListProps> = ({ 
  messages, 
  isLoading, 
  error 
}) => {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTo({
        top: scrollRef.current.scrollHeight,
        behavior: 'smooth'
      });
    }
  }, [messages, isLoading, error]);

  return (
    <div className="flex-1 overflow-y-auto p-4 space-y-6 scrollbar-thin scrollbar-thumb-gray-200" ref={scrollRef}>
      {messages.length === 0 && !isLoading && (
          <div className="h-full flex items-center justify-center text-gray-400 text-sm italic animate-in fade-in duration-500">
              No messages yet. Start a conversation!
          </div>
      )}
      {messages.map((m, index) => (
        <MessageItem 
          key={m.id} 
          message={m} 
          isLoading={isLoading}
          isLast={index === messages.length - 1}
          error={error}
        />
      ))}
      <div className="h-4" /> {/* Bottom spacing */}
    </div>
  );
};
