import React, { useState, useRef } from 'react';
import { CirclePlus, Send } from 'lucide-react';
import { CommandAutocomplete } from './CommandAutocomplete';
import { MentionAutocomplete } from './MentionAutocomplete';
import { FileChip } from './FileChip';
import type { Attachment } from '../types/chat';

interface ChatInputProps {
  value: string;
  onInputChange: (val: string) => void;
  onSendMessage: () => void;
  onNavigateHistory: (direction: 'up' | 'down') => void;
  isLoading: boolean;
  attachments: Attachment[];
  onAddAttachments: (files: FileList | File[]) => void;
  onRemoveAttachment: (id: string) => void;
}

export const ChatInput: React.FC<ChatInputProps> = ({ 
  value, 
  onInputChange, 
  onSendMessage, 
  onNavigateHistory,
  isLoading,
  attachments,
  onAddAttachments,
  onRemoveAttachment
}) => {
  const [autocompleteOpen, setAutocompleteOpen] = useState(false);
  const [mentionOpen, setMentionOpen] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleCommandSelect = (cmd: string) => {
    onInputChange(cmd + ' ');
    setAutocompleteOpen(false);
    inputRef.current?.focus();
  };

  const handleMentionSelect = (mention: string) => {
    const parts = value.split(/\s/);
    parts.pop(); // remove the '@...' part
    const newValue = parts.join(' ') + (parts.length > 0 ? ' ' : '') + mention + ' ';
    onInputChange(newValue);
    setMentionOpen(false);
    inputRef.current?.focus();
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      if ((autocompleteOpen && !value.includes(' ')) || mentionOpen) {
        // CMDK handles its own Enter for selection
      } else {
        e.preventDefault();
        onSendMessage();
        setAutocompleteOpen(false);
        setMentionOpen(false);
      }
    }
    if (e.key === 'Escape') {
      setAutocompleteOpen(false);
      setMentionOpen(false);
    }
    if (e.key === 'ArrowUp') { 
      if (!autocompleteOpen && !mentionOpen) {
        e.preventDefault(); 
        onNavigateHistory('up'); 
      }
    }
    if (e.key === 'ArrowDown') { 
      if (!autocompleteOpen && !mentionOpen) {
        e.preventDefault(); 
        onNavigateHistory('down'); 
      }
    }
  };

  return (
    <div className="p-4 bg-white border-t relative pt-6 pb-6">
      <div className="max-w-3xl mx-auto relative">
        <CommandAutocomplete 
          input={value} 
          onSelect={handleCommandSelect}
          isOpen={autocompleteOpen}
          setIsOpen={setAutocompleteOpen}
        />

        <MentionAutocomplete 
          input={value} 
          onSelect={handleMentionSelect}
          isOpen={mentionOpen}
          setIsOpen={setMentionOpen}
        />

        {attachments.length > 0 && (
          <div className="flex flex-wrap gap-2 mb-3 px-2">
            {attachments.map(a => (
              <FileChip key={a.id} attachment={a} onRemove={onRemoveAttachment} />
            ))}
          </div>
        )}

        <div className="flex items-center gap-1.5 border rounded-full px-2 py-1.5 bg-gray-50 shadow-sm focus-within:ring-2 focus-within:ring-black focus-within:border-black transition-all">
          <button 
            onClick={() => fileInputRef.current?.click()}
            className="flex items-center justify-center size-10 text-gray-500 hover:text-black transition-colors shrink-0"
            aria-label="Attach files"
          >
            <CirclePlus size={24} />
          </button>
          <input 
            type="file" 
            multiple 
            ref={fileInputRef} 
            onChange={(e) => e.target.files && onAddAttachments(e.target.files)}
            className="hidden" 
          />

          <textarea 
            ref={inputRef}
            value={value} 
            onChange={(e) => onInputChange(e.target.value)}
            onKeyDown={handleKeyDown}
            rows={1}
            className="flex-1 bg-transparent px-2 py-2 outline-none text-sm sm:text-base placeholder:text-gray-400 resize-none overflow-hidden max-h-[120px]" 
            placeholder="Message ..."
            disabled={isLoading && !value}
          />
          
          <button 
            onClick={onSendMessage} 
            disabled={isLoading || (!value.trim() && attachments.length === 0)}
            className="flex items-center justify-center size-10 bg-black text-white rounded-full hover:bg-gray-800 disabled:bg-gray-200 disabled:text-gray-400 transition-all shadow-md active:scale-95 shrink-0"
            aria-label="Send message"
          >
            <Send size={18} />
          </button>
        </div>
        <p className="text-[10px] text-center text-gray-400 mt-2">
            Nami may provide inaccurate information. Use with discretion.
        </p>
      </div>
    </div>
  );
};
