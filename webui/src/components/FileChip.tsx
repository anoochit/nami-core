import React from 'react';
import { X, File, Loader2, AlertCircle, CheckCircle2 } from 'lucide-react';
import { cn } from '../lib/utils';
import type { Attachment } from '../types/chat';

interface FileChipProps {
  attachment: Attachment;
  onRemove: (id: string) => void;
}

export const FileChip: React.FC<FileChipProps> = ({ attachment, onRemove }) => {
  return (
    <div className={cn(
      "flex items-center gap-2 px-3 py-1.5 rounded-full border text-xs font-medium animate-in fade-in zoom-in-95 duration-200",
      attachment.status === 'uploading' && "bg-blue-50 border-blue-200 text-blue-700",
      attachment.status === 'success' && "bg-green-50 border-green-200 text-green-700",
      attachment.status === 'error' && "bg-red-50 border-red-200 text-red-700"
    )}>
      <div className="shrink-0">
        {attachment.status === 'uploading' ? (
          <Loader2 size={14} className="animate-spin" />
        ) : attachment.status === 'success' ? (
          <CheckCircle2 size={14} />
        ) : attachment.status === 'error' ? (
          <AlertCircle size={14} />
        ) : (
          <File size={14} />
        )}
      </div>
      <span className="truncate max-w-[120px]" title={attachment.name}>{attachment.name}</span>
      <button 
        onClick={() => onRemove(attachment.id)}
        className="ml-1 p-0.5 hover:bg-black/5 rounded-full transition-colors"
        aria-label="Remove attachment"
      >
        <X size={14} />
      </button>
    </div>
  );
};
