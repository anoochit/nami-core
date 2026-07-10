import React, { useState } from "react";
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
      "text-xs px-2.5 py-1 bg-white border border-slate-200/80 rounded-lg text-slate-600 hover:text-slate-900 hover:bg-slate-50 hover:border-slate-300 transition-all duration-200 flex items-center gap-1.5 shadow-sm/5",
      className
    )}
    onClick={() => onClick?.(path)}
  >
    <FileText size={12} className="opacity-80" />
    <span className="truncate max-w-[180px]">Preview {path.split('/').pop()}</span>
  </button>
);

// STEP 2.2: Interactive Diff Block component to display code edits beautifully inside chat bubbles
const InteractiveDiffBlock = ({ content }: { content: string }) => {
  const [viewMode, setViewMode] = useState<'diff' | 'raw'>('diff');

  // Extract the filename if present
  let filename = "Suggested Code Patch";
  const lines = content.split('\n');
  for (const line of lines) {
    if (line.startsWith('+++ b/')) {
      filename = line.substring(6).trim();
      break;
    } else if (line.startsWith('+++ ')) {
      filename = line.substring(4).trim();
      break;
    }
  }

  // Parse lines for the diff table
  let leftLineNum = 0;
  let rightLineNum = 0;
  const parsedLines = lines.map((line) => {
    let type: 'hunk' | 'addition' | 'deletion' | 'context' | 'meta' = 'context';
    let leftNum: number | null = null;
    let rightNum: number | null = null;

    if (line.startsWith('@@ ')) {
      type = 'hunk';
      // Match @@ -leftStart,leftLen +rightStart,rightLen @@
      const match = line.match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
      if (match) {
        leftLineNum = parseInt(match[1]);
        rightLineNum = parseInt(match[2]);
      }
    } else if (line.startsWith('+') && !line.startsWith('+++ ')) {
      type = 'addition';
      rightNum = rightLineNum;
      rightLineNum++;
    } else if (line.startsWith('-') && !line.startsWith('--- ')) {
      type = 'deletion';
      leftNum = leftLineNum;
      leftLineNum++;
    } else if (line.startsWith('--- ') || line.startsWith('+++ ') || line.startsWith('diff ') || line.startsWith('index ')) {
      type = 'meta';
    } else {
      leftNum = leftLineNum;
      rightNum = rightLineNum;
      leftLineNum++;
      rightLineNum++;
    }

    return { text: line, type, leftNum, rightNum };
  });

  return (
    <div className="my-3 border border-slate-200/80 rounded-xl overflow-hidden bg-slate-950 font-mono text-[11px] shadow-md max-w-full">
      {/* Header with filename and tabs */}
      <div className="px-3.5 py-2 bg-slate-900 border-b border-slate-800 flex justify-between items-center text-slate-400 select-none flex-wrap gap-2">
        <div className="flex items-center gap-2 text-slate-200 font-medium">
          <span className="w-2 h-2 rounded-full bg-amber-500 animate-pulse" />
          <span className="truncate max-w-[280px]">{filename}</span>
        </div>
        <div className="flex bg-slate-950/80 p-0.5 rounded-lg border border-slate-800/80">
          <button
            onClick={() => setViewMode('diff')}
            className={cn(
              "px-2 py-1 rounded-md text-[10px] font-bold uppercase tracking-wider transition-all",
              viewMode === 'diff' ? "bg-slate-800 text-slate-100" : "text-slate-400 hover:text-slate-200"
            )}
          >
            Visual Diff
          </button>
          <button
            onClick={() => setViewMode('raw')}
            className={cn(
              "px-2 py-1 rounded-md text-[10px] font-bold uppercase tracking-wider transition-all",
              viewMode === 'raw' ? "bg-slate-800 text-slate-100" : "text-slate-400 hover:text-slate-200"
            )}
          >
            Raw Diff
          </button>
        </div>
      </div>

      {/* Body Content */}
      {viewMode === 'raw' ? (
        <pre className="p-3.5 text-slate-300 overflow-x-auto leading-relaxed max-h-96 scrollbar-thin select-text">
          <code>{content}</code>
        </pre>
      ) : (
        <div className="overflow-x-auto max-h-96 scrollbar-thin select-text bg-slate-950">
          <table className="w-full border-collapse leading-relaxed">
            <tbody>
              {parsedLines.map((l, idx) => {
                let rowBg = "";
                let codeColor = "text-slate-300";
                let numColor = "text-slate-600";
                let marker = " ";

                if (l.type === 'addition') {
                  rowBg = "bg-emerald-950/25 border-l-2 border-emerald-500";
                  codeColor = "text-emerald-300 font-medium";
                  numColor = "text-emerald-700";
                  marker = "+";
                } else if (l.type === 'deletion') {
                  rowBg = "bg-rose-950/25 border-l-2 border-rose-500";
                  codeColor = "text-rose-300 line-through opacity-85";
                  numColor = "text-rose-700";
                  marker = "-";
                } else if (l.type === 'hunk') {
                  rowBg = "bg-sky-950/15 text-sky-400/80 font-bold border-b border-sky-950/30";
                  codeColor = "text-sky-400";
                  numColor = "text-sky-800";
                } else if (l.type === 'meta') {
                  rowBg = "bg-slate-900/40 text-slate-400/70 italic";
                  codeColor = "text-slate-400/80";
                  numColor = "text-slate-600/40";
                }

                return (
                  <tr key={idx} className={cn("hover:bg-slate-900/30 transition-colors", rowBg)}>
                    <td className={cn("w-9 text-right pr-2 select-none border-r border-slate-900 font-mono text-[10px]", numColor)}>
                      {l.leftNum !== null ? l.leftNum : ""}
                    </td>
                    <td className={cn("w-9 text-right pr-2 select-none border-r border-slate-900 font-mono text-[10px]", numColor)}>
                      {l.rightNum !== null ? l.rightNum : ""}
                    </td>
                    <td className="w-5 text-center select-none font-semibold pl-1.5 pr-0.5 opacity-40">
                      {marker !== ' ' ? marker : ''}
                    </td>
                    <td className={cn("px-3 py-0.5 whitespace-pre font-mono text-left", codeColor)}>
                      {l.text.startsWith('+') || l.text.startsWith('-') ? l.text.substring(1) : l.text}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

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
                "font-mono px-1.5 py-0.5 rounded text-xs mx-0.5 align-baseline transition-all duration-200 border",
                isAgent 
                  ? "text-slate-900 bg-slate-100 border-slate-200 hover:bg-slate-200/60" 
                  : "text-white bg-white/10 border-white/10 hover:bg-white/20"
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
          <div className="group/file my-2.5 p-2.5 border border-slate-200/60 rounded-xl bg-slate-50/50 hover:bg-slate-50 flex items-center justify-between transition-colors shadow-sm/5">
            <span className="font-mono text-xs text-slate-700 font-medium">{content}</span>
            <FilePreviewButton path={content} onClick={onPreviewFile} />
          </div>
        );
      }

      // Render interactive diff blocks
      const isDiff = className === "language-diff" || className === "language-patch" || content.startsWith("@@ ") || content.includes("\n+");
      if (!inline && isDiff) {
        return <InteractiveDiffBlock content={content} />;
      }

      return (
        <code className={cn(className, "font-mono text-xs")} {...props}>
          {children}
        </code>
      );
    },
    p: ({ children }: any) => {
        return (
            <p className="leading-relaxed">
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
        "flex gap-3.5 animate-in fade-in duration-300 px-4 sm:px-6 py-2",
        message.sender === "agent" ? "justify-start" : "justify-end",
      )}
    >
      {message.sender === "agent" && (
        <div className="w-8 h-8 flex items-center justify-center bg-slate-50 border border-slate-200 text-slate-600 rounded-lg shrink-0 shadow-sm">
          <Bot size={15} />
        </div>
      )}

      <div
        className={cn(
          "p-4 rounded-2xl max-w-[85%] sm:max-w-[78%] flex flex-col gap-3 transition-all duration-300 shadow-sm",
          message.sender === "agent"
            ? "bg-white border border-slate-100 text-slate-800 rounded-tl-none"
            : "bg-slate-900 text-slate-50 rounded-tr-none shadow-[0_4px_12px_rgba(15,23,42,0.08)]",
        )}
      >
        {message.attachments && message.attachments.length > 0 && (
          <div className="flex flex-wrap gap-1.5">
            {message.attachments.map(a => {
              const path = a.path || a.name;
              return isAgent ? (
                <FilePreviewButton 
                  key={a.id} 
                  path={path} 
                  onClick={onPreviewFile}
                  className="text-[10px]"
                />
              ) : (
                <div
                  key={a.id}
                  className="text-[10px] px-2.5 py-1 bg-white/10 border border-white/5 rounded-lg text-slate-200 flex items-center gap-1.5 shadow-sm/5 font-sans"
                >
                  <FileText size={11} className="opacity-80 text-slate-300" />
                  <span className="truncate max-w-[180px]">{path.split('/').pop()}</span>
                </div>
              );
            })}
          </div>
        )}

        {message.toolCall && (
          <div className="mb-1">
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
                <div className="mt-1.5 flex gap-2">
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
            "prose prose-sm max-w-none prose-p:m-0 prose-pre:bg-slate-950 prose-pre:text-slate-100 prose-pre:border prose-pre:border-slate-800 prose-pre:rounded-xl prose-pre:p-3.5",
            !isAgent && "prose-invert prose-p:text-slate-200"
        )}>
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={MarkdownComponents}
          >
            {message.text || (isLoading && isLast && isAgent ? "Thinking..." : "")}
          </ReactMarkdown>
        </div>

        {isLoading && isLast && isAgent && !message.text && (
          <div className="flex gap-1.5 mt-2 self-start bg-slate-50 border border-slate-100 px-3 py-2 rounded-lg">
            <div className="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce"></div>
            <div className="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce [animation-delay:0.2s]"></div>
            <div className="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce [animation-delay:0.4s]"></div>
          </div>
        )}

        {isLoading && isLast && isAgent && message.text && (
          <div className="w-3.5 h-3.5 mt-2 border-2 border-slate-200 border-t-slate-800 rounded-full animate-spin"></div>
        )}

        {!isLoading && error && isLast && isAgent && (
          <div className="mt-3 p-3 bg-red-50/60 border border-red-100 rounded-xl text-red-800 overflow-hidden">
            <div className="flex items-center gap-2 mb-1.5 font-display font-semibold text-xs uppercase tracking-wider">
              <AlertCircle size={14} className="text-red-500" />
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
