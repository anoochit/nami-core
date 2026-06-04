import { cn } from './lib/utils';
import { ThreadView } from './components/ThreadView';
import { FilePreview } from './components/FilePreview';
import { FileExplorer } from './components/FileExplorer';
import { useChat } from './hooks/useChat';
import { ServerStatusIndicator } from './components/ServerStatusIndicator';
import { Group, Panel, Separator } from 'react-resizable-panels';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs';
import {  FolderTreeIcon,  } from 'lucide-react';

export default function App() {
  const {
    activeThread,
    input,
    sidebarOpen,
    activePreviewPath,
    activePreviewWikiPath,
    isLoading,
    error,
    setInput,
    setSidebarOpen,
    setActivePreviewPath,
    setActivePreviewWikiPath,
    sendMessage,
    createNewThread,
    navigateHistory,
    clearMessages,
    attachments,
    addAttachments,
    removeAttachment,
    messageQueue
  } = useChat();

  const activePreview = activePreviewPath || activePreviewWikiPath;

  return (
    <div className="flex h-screen bg-white overflow-hidden">
      <div className={cn("w-64 border-r bg-gray-50 flex flex-col transition-all duration-300 shrink-0", !sidebarOpen && "-ml-64")}>
        <Tabs defaultValue="files" className="flex-1 flex flex-col overflow-hidden">
          <TabsList className="w-full">
            <TabsTrigger value="files" className="flex-1 justify-start pb-4 pt-4"><FolderTreeIcon/>Files</TabsTrigger>
          </TabsList>
          <TabsContent value="files" className="flex-1 overflow-hidden">
             <FileExplorer onOpenFile={setActivePreviewPath} />
          </TabsContent>
        </Tabs>
        <div className="p-2 border-t flex justify-between items-center">
            <span className="font-bold text-sm">Server status</span>
            <ServerStatusIndicator />
        </div>
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
              onPreviewFile={setActivePreviewPath}
              onPreviewWiki={setActivePreviewWikiPath}
              onClear={clearMessages}
              onNewThread={createNewThread}
              isLoading={isLoading}
              error={error ?? undefined}
              attachments={attachments}
              onAddAttachments={addAttachments}
              onRemoveAttachment={removeAttachment}
              queueCount={messageQueue.length}
            />
          </Panel>
          
          {activePreview && (
            <>
              <Separator className="w-1 bg-gray-100 hover:bg-blue-400 transition-colors cursor-col-resize" />
              <Panel minSize={25} defaultSize={40}>
                <FilePreview 
                  path={activePreview} 
                  isWiki={!!activePreviewWikiPath}
                  onClose={() => {
                    setActivePreviewPath(null);
                    setActivePreviewWikiPath(null);
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
