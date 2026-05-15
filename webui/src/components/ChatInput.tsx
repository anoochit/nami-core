import React, { useState, useRef } from 'react';
import { CommandAutocomplete } from './CommandAutocomplete';

interface ChatInputProps {
  value: string;
  onInputChange: (val: string) => void;
  onSendMessage: () => void;
  onNavigateHistory: (direction: 'up' | 'down') => void;
  isLoading: boolean;
}

export const ChatInput: React.FC<ChatInputProps> = ({ 
  value, 
  onInputChange, 
  onSendMessage, 
  onNavigateHistory,
  isLoading
}) => {
  const [autocompleteOpen, setAutocompleteOpen] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const handleCommandSelect = (cmd: string) => {
    onInputChange(cmd + ' ');
    setAutocompleteOpen(false);
    inputRef.current?.focus();
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      if (autocompleteOpen && !value.includes(' ')) {
        // CMDK handles its own Enter for selection
      } else {
        e.preventDefault();
        onSendMessage();
        setAutocompleteOpen(false);
      }
    }
    if (e.key === 'Escape') {
      setAutocompleteOpen(false);
    }
    if (e.key === 'ArrowUp') { 
      if (!autocompleteOpen) {
        e.preventDefault(); 
        onNavigateHistory('up'); 
      }
    }
    if (e.key === 'ArrowDown') { 
      if (!autocompleteOpen) {
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
        <div className="flex gap-2 border  rounded-3xl p-1 bg-gray-50 shadow-sm focus-within:ring-2 focus-within:ring-black focus-within:border-black transition-all">
          <textarea 
            ref={inputRef}
            value={value} 
            onChange={(e) => onInputChange(e.target.value)}
            onKeyDown={handleKeyDown}
            rows={1}
            className="flex-1 bg-transparent px-4 py-2.5 outline-none text-sm sm:text-base placeholder:text-gray-400 resize-none overflow-hidden" 
            placeholder="Message ..."
            disabled={isLoading && !value}
          />
          {/* <button 
            onClick={onSendMessage} 
            disabled={isLoading || !value.trim()}
            className="w-10 h-10 p-0 grid place-items-center bg-black text-white rounded-full hover:bg-gray-800 disabled:bg-gray-200 disabled:text-gray-400 transition-all shadow-md active:scale-95 shrink-0 overflow-hidden"
            aria-label="Send message"
          >
            <Send size={18} className="translate-x-[-1px] translate-y-[1px]" />
          </button> */}
        </div>
        <p className="text-[10px] text-center text-gray-400 mt-2">
            Nami may provide inaccurate information. Use with discretion.
        </p>
      </div>
    </div>
  );
};
