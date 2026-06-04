import React from "react";
import { Bot, AlertCircle, FileText } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { cn, formatError } from "../lib/utils";
import { ToolAccordion } from "./ToolAccordion";
import type { Message } from "../types/chat";

interface MessageItemProps {
  message: Message;
  isLoading: boolean;
  isLast: boolean;
  error?: string;
  onPreviewFile?: (path: string) => void;
  onPreviewWiki?: (path: string) => void;
}

const FilePreviewButton = ({ 
  path, 
  onClick,
  className 
}: { 
  path: string; 
  onClick?: (path: string) => void;
  className?: string 
}) => (
  <button
    className={cn(
      "text-xs px-2 py-1 bg-white border rounded hover:bg-gray-50 transition-colors flex items-center gap-1 shadow-sm",
      className
    )}
    onClick={() => onClick?.(path)}
  >
    <FileText size={12} />
    Preview {path}
  </button>
);

export const MessageItem: React.FC<MessageItemProps> = ({
  message,
  isLoading,
  isLast,
  error,
  onPreviewFile,
  onPreviewWiki,
}) => {
  const isAgent = message.sender === "agent";

  const renderTextWithMentions = (text: string) => {
    if (!text.includes('@')) return text;

    const mentionRegex = /(@[a-zA-Z0-9._\-/]+)/g;
    const parts = text.split(mentionRegex);
    
    return parts.map((part, i) => {
      if (part.startsWith('@')) {
        const path = part.slice(1);
        const isWiki = path.startsWith('wiki/');
        const displayPath = isWiki ? path.slice(5) : path;
        
        return (
          <button
            key={i}
            onClick={() => isWiki ? onPreviewWiki?.(displayPath) : onPreviewFile?.(path)}
            className={cn(
                "font-mono px-1 rounded mx-0.5 align-baseline transition-colors text-sm",
                isAgent ? "text-blue-600 bg-blue-50 hover:bg-blue-100" : "text-blue-300 bg-blue-900/30 hover:bg-blue-900/50"
            )}
          >
            {part}
          </button>
        );
      }
      return part;
    });
  };

  const MarkdownComponents = {
    code: ({ inline, className, children, ...props }: any) => {
      const content = String(children).replace(/\n$/, "");

      if (inline && onPreviewFile && /^[a-zA-Z0-9._\-/]+\.[a-zA-Z0-9]+$/.test(content)) {
        return (
          <div className="group/file my-2 p-2 border rounded bg-gray-50 hover:bg-gray-100 flex items-center justify-between transition-colors">
            <span className="font-mono text-sm">{content}</span>
            <FilePreviewButton path={content} onClick={onPreviewFile} />
          </div>
        );
      }

      return (
        <code className={className} {...props}>
          {children}
        </code>
      );
    },
    p: ({ children }: any) => {
        return (
            <p>
                {React.Children.map(children, child => 
                    typeof child === 'string' ? renderTextWithMentions(child) : child
                )}
            </p>
        );
    }
  };

  return (
    <div
      className={cn(
        "flex gap-3 animate-in fade-in duration-300",
        message.sender === "agent" ? "justify-start" : "justify-end",
      )}
    >
      {message.sender === "agent" && (
        <div className="w-8 h-8 flex items-center justify-center bg-gray-800 text-white rounded-full shrink-0 shadow-sm">
          <Bot size={16} />
        </div>
      )}

      <div
        className={cn(
          "p-3 rounded-2xl max-w-[85%] sm:max-w-[75%] flex flex-col gap-2",
          message.sender === "agent"
            ? "bg-gray-100 text-gray-800 rounded-tl-sm"
            : "bg-black text-white rounded-tr-sm shadow-sm",
        )}
      >
        {message.attachments && message.attachments.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {message.attachments.map(a => (
              <FilePreviewButton 
                key={a.id} 
                path={a.path || a.name} 
                onClick={onPreviewFile}
                className={cn(
                    "text-[10px]",
                    !isAgent && "bg-white/10 border-white/20 text-white hover:bg-white/20"
                )} 
              />
            ))}
          </div>
        )}

        {message.toolCall && (
          <div className="mb-2">
            <ToolAccordion
              title={`Tool: ${message.toolCall.name}`}
              args={message.toolCall.args}
              result={message.toolCall.result}
            />
            {message.toolCall.status === "complete" &&
              message.toolCall.result !== null &&
              typeof message.toolCall.result === "object" &&
              ("filename" in message.toolCall.result ||
                "path" in message.toolCall.result) && (
                <div className="mt-1 flex gap-2">
                  <FilePreviewButton
                    path={
                      (message.toolCall.result as any).filename ||
                      (message.toolCall.result as any).path
                    }
                    onClick={onPreviewFile}
                  />
                </div>
              )}
          </div>
        )}

        <div className={cn(
            "prose prose-sm sm:prose-base max-w-none prose-p:m-0 prose-pre:bg-gray-800 prose-pre:text-gray-100",
            !isAgent && "prose-invert"
        )}>
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={MarkdownComponents}
          >
            {message.text || (isLoading && isLast && isAgent ? "Thinking..." : "")}
          </ReactMarkdown>
        </div>

        {isLoading && isLast && isAgent && !message.text && (
          <div className="flex gap-1 mt-2">
            <div className="w-1.5 h-1.5 bg-gray-400 rounded-full animate-bounce"></div>
            <div className="w-1.5 h-1.5 bg-gray-400 rounded-full animate-bounce [animation-delay:0.2s]"></div>
            <div className="w-1.5 h-1.5 bg-gray-400 rounded-full animate-bounce [animation-delay:0.4s]"></div>
          </div>
        )}

        {isLoading && isLast && isAgent && message.text && (
          <div className="w-3 h-3 mt-2 border-2 border-gray-300 border-t-black rounded-full animate-spin"></div>
        )}

        {!isLoading && error && isLast && isAgent && (
          <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-800 overflow-hidden">
            <div className="flex items-center gap-2 mb-1 font-bold text-xs uppercase tracking-tight">
              <AlertCircle size={14} />
              <span>System Error</span>
            </div>
            <div className="prose prose-xs prose-red max-w-none">
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                components={MarkdownComponents}
              >
                {formatError(error)}
              </ReactMarkdown>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
