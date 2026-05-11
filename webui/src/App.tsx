import { cn } from './lib/utils';
import { ThreadList } from './components/ThreadList';
import { ThreadView } from './components/ThreadView';
import { useChat } from './hooks/useChat';

export default function App() {
  const {
    threads,
    activeThread,
    activeThreadId,
    input,
    sidebarOpen,
    isLoading,
    setActiveThreadId,
    setInput,
    setSidebarOpen,
    sendMessage,
    createNewThread
  } = useChat();

  return (
    <div className="flex h-screen bg-white">
      <div className={cn("w-64 border-r bg-gray-50 flex flex-col transition-all duration-300", !sidebarOpen && "-ml-64")}>
        <ThreadList 
            threads={threads} 
            activeThreadId={activeThreadId} 
            onSelectThread={setActiveThreadId} 
            onNewThread={createNewThread} 
        />
      </div>

      <ThreadView 
        thread={activeThread}
        input={input}
        sidebarOpen={sidebarOpen}
        onToggleSidebar={() => setSidebarOpen(!sidebarOpen)}
        onInputChange={setInput}
        onSendMessage={sendMessage}
        isLoading={isLoading}
      />
    </div>
  );
}
