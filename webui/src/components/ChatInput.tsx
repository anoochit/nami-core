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
  queueCount?: number;
}

export const ChatInput: React.FC<ChatInputProps> = ({ 
  value, 
  onInputChange, 
  onSendMessage, 
  onNavigateHistory,
  isLoading,
  attachments,
  onAddAttachments,
  onRemoveAttachment,
  queueCount = 0
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
    <div className="p-4 bg-white border-t border-slate-100 relative shrink-0">
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
          <div className="flex flex-wrap gap-1.5 mb-3 px-1.5">
            {attachments.map(a => (
              <FileChip key={a.id} attachment={a} onRemove={onRemoveAttachment} />
            ))}
          </div>
        )}

        <div className="flex items-center gap-1.5 border border-slate-200/80 rounded-2xl px-2 py-1.5 bg-slate-50/50 shadow-sm/5 focus-within:ring-1 focus-within:ring-slate-300 focus-within:border-slate-400 focus-within:bg-white transition-all duration-300">
          <button 
            onClick={() => fileInputRef.current?.click()}
            className="flex items-center justify-center size-9 text-slate-400 hover:text-slate-800 hover:bg-slate-100 rounded-xl transition-all duration-200 shrink-0"
            aria-label="Attach files"
          >
            <CirclePlus size={20} />
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
            className="flex-1 bg-transparent px-2.5 py-1.5 outline-none text-sm placeholder:text-slate-400/80 resize-none overflow-hidden max-h-[120px] font-sans text-slate-800 leading-relaxed" 
            placeholder={isLoading ? "Queueing message..." : "Message ... (Type / for commands, @ for files)"}
          />
          
          <button 
            onClick={onSendMessage} 
            disabled={!value.trim() && attachments.length === 0}
            className="flex items-center justify-center size-9 bg-slate-900 text-slate-50 rounded-xl hover:bg-slate-800 disabled:bg-slate-100 disabled:text-slate-300 transition-all duration-200 shadow active:scale-[0.98] shrink-0 relative"
            aria-label="Send message"
          >
            <Send size={15} />
            {queueCount > 0 && (
              <span className="absolute -top-1 -right-1 bg-blue-500 text-white text-[8px] font-bold px-1.5 py-0.5 rounded-full ring-2 ring-white animate-in zoom-in duration-300">
                {queueCount}
              </span>
            )}
          </button>
        </div>
        
        <p className="text-[9px] uppercase tracking-wider font-display font-medium text-center text-slate-400 mt-2.5">
          Nami may provide inaccurate information. Use with discretion.
        </p>
      </div>
    </div>
  );
};
