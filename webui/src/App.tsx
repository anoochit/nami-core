import { cn } from './lib/utils';
import { ThreadList } from './components/ThreadList';
import { ThreadView } from './components/ThreadView';
import { PreviewPane } from './components/PreviewPane';
import { useChat } from './hooks/useChat';
import { ServerStatusIndicator } from './components/ServerStatusIndicator';
import { Group, Panel, Separator } from 'react-resizable-panels';

export default function App() {
  const {
    threads,
    activeThread,
    activeThreadId,
    input,
    sidebarOpen,
    previewPath,
    previewWikiPath,
    isLoading,
    error,
    setActiveThreadId,
    setInput,
    setSidebarOpen,
    setPreviewPath,
    setPreviewWikiPath,
    sendMessage,
    createNewThread,
    navigateHistory
  } = useChat();

  const activePreview = previewPath || previewWikiPath;

  return (
    <div className="flex h-screen bg-white overflow-hidden">
      {/* ... sidebar ... */}
      <div className={cn("w-64 border-r bg-gray-50 flex flex-col transition-all duration-300 shrink-0", !sidebarOpen && "-ml-64")}>
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

      <div className="flex-1 flex overflow-hidden">
        <Group orientation="horizontal">
          <Panel defaultSize={activePreview ? 60 : 100} minSize={30}>
            <ThreadView 
              thread={activeThread}
              input={input}
              sidebarOpen={sidebarOpen}
              onToggleSidebar={() => setSidebarOpen(!sidebarOpen)}
              onInputChange={setInput}
              onSendMessage={sendMessage}
              onNavigateHistory={navigateHistory}
              onPreviewFile={setPreviewPath}
              onPreviewWiki={setPreviewWikiPath}
              isLoading={isLoading}
              error={error ?? undefined}
            />
          </Panel>
          
          {activePreview && (
            <>
              <Separator className="w-1 bg-gray-100 hover:bg-blue-400 transition-colors cursor-col-resize" />
              <Panel minSize="25%" defaultSize="40%">
                <PreviewPane 
                  path={activePreview} 
                  isWiki={!!previewWikiPath}
                  onClose={() => {
                    setPreviewPath(null);
                    setPreviewWikiPath(null);
                  }} 
                />
              </Panel>
            </>
          )}
        </Group>
      </div>
    </div>
  );
}
