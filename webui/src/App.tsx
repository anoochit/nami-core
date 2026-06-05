import { cn } from './lib/utils';
import { ThreadView } from './components/ThreadView';
import { FilePreview } from './components/FilePreview';
import { FileExplorer } from './components/FileExplorer';
import { useChat } from './hooks/useChat';
import { ServerStatusIndicator } from './components/ServerStatusIndicator';
import { Group, Panel, Separator } from 'react-resizable-panels';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs';
import { FolderTreeIcon, MessageSquare, BookOpen } from 'lucide-react';
import { Titlebar } from './components/Titlebar';
import { WikiExplorer } from './components/WikiExplorer';


export default function App() {
  const {
    activeThread,
    input,
    sidebarOpen,
    activePreviewPath,
    activePreviewWikiPath,
    isLoading,
    error,
    sessionHistory,
    loadSession,
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
    <div className="flex flex-col h-screen bg-slate-50 font-sans overflow-hidden antialiased">
      <Titlebar />
      <div className="flex flex-1 overflow-hidden relative">
      {/* Premium Sidebar */}
      <div className={cn(
        "w-72 border-r border-slate-200/60 bg-white flex flex-col transition-all duration-300 shrink-0 z-20 shadow-[1px_0_10px_rgba(0,0,0,0.01)]", 
        !sidebarOpen && "-ml-72"
      )}>
        <Tabs defaultValue="files" className="flex-1 flex flex-col overflow-hidden">
          <TabsList className="w-full bg-slate-50 p-1 border-b border-slate-100 flex gap-1 h-12 rounded-none">
            <TabsTrigger value="files" className="flex-1 justify-center py-2 text-xs font-display transition-all duration-200">
              <FolderTreeIcon size={14} className="mr-1.5 opacity-80" />
              Files
            </TabsTrigger>
            <TabsTrigger value="wiki" className="flex-1 justify-center py-2 text-xs font-display transition-all duration-200">
              <BookOpen size={14} className="mr-1.5 opacity-80" />
              Wiki
            </TabsTrigger>
            <TabsTrigger value="sessions" className="flex-1 justify-center py-2 text-xs font-display transition-all duration-200">
              <MessageSquare size={14} className="mr-1.5 opacity-80" />
              Sessions
            </TabsTrigger>
          </TabsList>
          
          <TabsContent value="files" className="flex-1 overflow-hidden bg-white">
             <FileExplorer onOpenFile={setActivePreviewPath} />
          </TabsContent>

          <TabsContent value="wiki" className="flex-1 overflow-hidden bg-white">
             <WikiExplorer onOpenFile={setActivePreviewPath} />
          </TabsContent>
          
          <TabsContent value="sessions" className="flex-1 overflow-hidden p-3 bg-white">
            <div className="font-display font-semibold text-[10px] text-slate-400 uppercase tracking-wider mb-3 px-1">
              Recent Sessions
            </div>
            <div className="overflow-y-auto h-[calc(100vh-140px)] space-y-1.5 pr-1 scrollbar-thin">
              {sessionHistory.length === 0 ? (
                <div className="text-center text-xs text-slate-400 py-8 italic font-sans">
                  No active sessions yet.
                </div>
              ) : (
                sessionHistory.map(session => (
                  <div key={session.session_id} 
                       onClick={() => loadSession(session.session_id)}
                       className="group p-2.5 rounded-lg border border-slate-100 bg-slate-50/30 text-sm cursor-pointer hover:bg-slate-50 hover:border-slate-200/80 transition-all duration-200 shadow-sm/5">
                     <div className="font-mono text-xs font-medium text-slate-700 truncate group-hover:text-slate-900 transition-colors">
                       {session.session_id.substring(0, 12)}...
                     </div>
                     <div className="text-[10px] text-slate-400 mt-1 flex justify-between items-center">
                       <span>{new Date(session.created_at).toLocaleDateString()}</span>
                       <span className="opacity-0 group-hover:opacity-100 transition-opacity text-slate-500 font-sans text-[10px] font-medium">Load &rarr;</span>
                     </div>
                  </div>
                ))
              )}
            </div>
          </TabsContent>
        </Tabs>
        
        <div className="p-3 bg-slate-50/50 border-t border-slate-100 flex justify-between items-center h-12 shrink-0">
            <span className="font-display font-semibold text-[10px] text-slate-400 uppercase tracking-wider">
              Server Connection
            </span>
            <ServerStatusIndicator />
        </div>
      </div>

      {/* Main Workspace Panels */}
      <div className="flex-1 flex overflow-hidden">
        <Group orientation="horizontal">
          <Panel defaultSize={activePreview ? 55 : 100} minSize={30}>
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
              <Separator className="w-[1.5px] bg-slate-200/80 hover:bg-slate-400 transition-colors cursor-col-resize relative z-10" />
              <Panel minSize={25} defaultSize={45}>
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
    </div>
  );
}
