import React from "react";
import { MessageSquare, Plus } from "lucide-react";
import { cn } from "../lib/utils";

interface Thread {
  id: string;
  title: string;
  messages: any[];
  sessionId?: string;
}

interface ThreadListProps {
  threads: Thread[];
  activeThreadId: string;
  onSelectThread: (id: string) => void;
  onNewThread: () => void;
}

export const ThreadList: React.FC<ThreadListProps> = ({
  threads,
  activeThreadId,
  onSelectThread,
  onNewThread,
}) => {
  return (
    <div className="flex flex-col h-full ">
      <div className="p-2">
        <button
          onClick={onNewThread}
          className="w-full flex items-center justify-center gap-2 bg-black text-white p-2 rounded-md hover:bg-gray-800"
        >
          <Plus size={18} /> New Chat
        </button>
      </div>
      <div className="flex-1 overflow-y-auto p-2  ">
        {threads.map((thread) => (
          <div
            key={thread.id}
            onClick={() => onSelectThread(thread.id)}
            className={cn(
              "p-3 rounded-md cursor-pointer flex items-center gap-2 hover:bg-gray-200 mb-1",
              activeThreadId === thread.id && "bg-gray-200",
            )}
          >
            <MessageSquare size={16} />
            <span className="truncate">{thread.title}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
