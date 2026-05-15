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

export const MessageItem: React.FC<MessageItemProps> = ({
  message,
  isLoading,
  isLast,
  error,
  onPreviewFile,
}) => {
  const isAgent = message.sender === "agent";

  // Helper to detect if a string is likely a filename or wiki page
  const isFileName = (text: string) =>
    /^[a-zA-Z0-9._\-\/]+\.[a-zA-Z0-9]+$/.test(text);

  const FilePreviewButton = ({ path, className }: { path: string; className?: string }) => (
    <button
      className={cn(
        "text-xs px-2 py-1 bg-white border rounded hover:bg-gray-50 transition-colors flex items-center gap-1 shadow-sm",
        className
      )}
      onClick={() => onPreviewFile?.(path)}
    >
      <FileText size={12} />
      Preview {path}
    </button>
  );

  const MarkdownComponents = {
    code: ({ node, inline, className, children, ...props }: any) => {
      const content = String(children).replace(/\n$/, "");

      if (inline && onPreviewFile && isFileName(content)) {
        return (
          <div className="group/file my-2 p-2 border rounded bg-gray-50 hover:bg-gray-100 flex items-center justify-between transition-colors">
            <span className="font-mono text-sm">{content}</span>
            <FilePreviewButton path={content} />
          </div>
        );
      }

      return (
        <code className={className} {...props}>
          {children}
        </code>
      );
    },
  };

  return (
    <div
      className={cn(
        "flex gap-3 animate-in fade-in duration-300",
        isAgent ? "justify-start" : "justify-end",
      )}
    >
      {isAgent && (
        <div className="w-8 h-8 flex items-center justify-center bg-gray-800 text-white rounded-full shrink-0 shadow-sm">
          <Bot size={16} />
        </div>
      )}

      <div
        className={cn(
          "p-3 rounded-2xl max-w-[85%] sm:max-w-[75%]",
          isAgent
            ? "bg-gray-100 text-gray-800 rounded-tl-sm"
            : "bg-black text-white rounded-tr-sm shadow-sm",
        )}
      >
        {message.toolCall && (
          <div className="mb-2">
            <ToolAccordion
              title={`Tool: ${message.toolCall.name}`}
              args={message.toolCall.args}
              result={message.toolCall.result}
            />
            {message.toolCall.status === "complete" &&
              typeof message.toolCall.result === "object" &&
              message.toolCall.result !== null &&
              ("filename" in message.toolCall.result ||
                "path" in message.toolCall.result) && (
                <div className="mt-2 flex gap-2">
                  <FilePreviewButton
                    path={
                      (message.toolCall.result as any).filename ||
                      (message.toolCall.result as any).path
                    }
                  />
                </div>
              )}
          </div>
        )}

        {isAgent ? (
          <div className="prose prose-sm sm:prose-base max-w-none prose-p:m-0 prose-pre:bg-gray-800 prose-pre:text-gray-100">
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              components={MarkdownComponents}
            >
              {message.text || (isLoading && isLast ? "Thinking..." : "")}
            </ReactMarkdown>
          </div>
        ) : (
          <div className="whitespace-pre-wrap text-sm sm:text-base leading-relaxed">
            {message.text}
          </div>
        )}

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
