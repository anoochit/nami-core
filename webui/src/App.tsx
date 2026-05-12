import { cn } from './lib/utils';
import { ThreadList } from './components/ThreadList';
import { ThreadView } from './components/ThreadView';
import { useChat } from './hooks/useChat';
import { ServerStatusIndicator } from './components/ServerStatusIndicator';

export default function App() {
  const {
    threads,
    activeThread,
    activeThreadId,
    input,
    sidebarOpen,
    isLoading,
    error,
    setActiveThreadId,
    setInput,
    setSidebarOpen,
    sendMessage,
    createNewThread,
    navigateHistory
  } = useChat();

  return (
    <div className="flex h-screen bg-white">
      <div className={cn("w-64 border-r bg-gray-50 flex flex-col transition-all duration-300", !sidebarOpen && "-ml-64")}>
        <div className="p-2 border-b flex justify-between items-center">
            <span className="font-bold text-sm">Server status</span>
            <ServerStatusIndicator />
        </div>
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
        onNavigateHistory={navigateHistory}
        isLoading={isLoading}
        error={error ?? undefined}
      />
    </div>
  );
}
