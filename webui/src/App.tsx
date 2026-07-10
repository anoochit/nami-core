import { useState } from 'react';
import { cn } from './lib/utils';
import { ThreadView } from './components/ThreadView';
import { FilePreview } from './components/FilePreview';
import { FileExplorer } from './components/FileExplorer';
import { useChat } from './hooks/useChat';
// import { ServerStatusIndicator } from './components/ServerStatusIndicator';
import { Group, Panel, Separator } from 'react-resizable-panels';
import { FolderTreeIcon, MessageSquare, BookOpen, CalendarRange, ListTodo, Trash2, Plus } from 'lucide-react';
import { Titlebar } from './components/Titlebar';
import { WikiExplorer } from './components/WikiExplorer';
import { SchedulerExplorer } from './components/SchedulerExplorer';
import { TodoExplorer } from './components/TodoExplorer';


export default function App() {
  const {
    threads,
    activeThread,
    activeThreadId,
    input,
    sidebarOpen,
    activePreviewPath,
    activePreviewWikiPath,
    isLoading,
    error,
    setActiveThreadId,
    deleteThread,
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

  const [activeTab, setActiveTab] = useState<'files' | 'wiki' | 'sessions' | 'scheduler' | 'todos'>('files');

  const handleTabClick = (tab: 'files' | 'wiki' | 'sessions' | 'scheduler' | 'todos') => {
    if (activeTab === tab) {
      setSidebarOpen(!sidebarOpen);
    } else {
      setActiveTab(tab);
      setSidebarOpen(true);
    }
  };

  const activePreview = activePreviewPath || activePreviewWikiPath;

  return (
    <div className="flex flex-col h-screen bg-slate-50 font-sans overflow-hidden antialiased">
      <Titlebar />
      <div className="flex flex-1 overflow-hidden relative flex-col-reverse lg:flex-row">
      
      {/* Slim Vertical Icon Sidebar (Light on desktop, bottom navigation bar on mobile) */}
      <div className="w-full h-16 bg-white border-t border-slate-200/60 flex flex-row items-center justify-around px-4 shrink-0 z-25 shadow-[0_-2px_10px_rgba(0,0,0,0.02)] lg:w-16 lg:h-full lg:flex-col lg:border-t-0 lg:border-r lg:justify-between lg:py-4 lg:px-0 lg:shadow-none">
        <div className="flex flex-row lg:flex-col gap-2 lg:gap-4 w-full items-center justify-around lg:justify-start">
          {/* New Chat Button */}
          <button
            onClick={() => {
              createNewThread();
              setActiveTab('sessions');
              setSidebarOpen(true);
            }}
            className="p-3 rounded-full bg-sky-500 hover:bg-sky-600 active:scale-95 text-white transition-all duration-200 shadow-sm shadow-sky-500/20 hover:shadow-sky-500/35 group flex items-center justify-center shrink-0"
            title="Create New Chat"
          >
            <Plus size={20} className="transition-transform duration-200 group-hover:rotate-90" />
          </button>

          {/* Files Icon */}
          <button
            onClick={() => handleTabClick('files')}
            className={cn(
              "relative p-3 rounded-xl transition-all duration-200 group flex items-center justify-center",
              activeTab === 'files' && sidebarOpen
                ? "text-sky-600 bg-sky-50/60"
                : "text-slate-500 hover:text-slate-800 hover:bg-slate-100/50"
            )}
            title="File Explorer"
          >
            {activeTab === 'files' && sidebarOpen && (
              <>
                <div className="absolute left-0 top-3 bottom-3 w-0.5 bg-sky-500 rounded-r lg:block hidden" />
                <div className="absolute top-0 left-3 right-3 h-0.5 bg-sky-500 rounded-b lg:hidden block" />
              </>
            )}
            <FolderTreeIcon size={20} />
          </button>

          {/* Wiki Icon */}
          <button
            onClick={() => handleTabClick('wiki')}
            className={cn(
              "relative p-3 rounded-xl transition-all duration-200 group flex items-center justify-center",
              activeTab === 'wiki' && sidebarOpen
                ? "text-sky-600 bg-sky-50/60"
                : "text-slate-500 hover:text-slate-800 hover:bg-slate-100/50"
            )}
            title="Wiki Pages"
          >
            {activeTab === 'wiki' && sidebarOpen && (
              <>
                <div className="absolute left-0 top-3 bottom-3 w-0.5 bg-sky-500 rounded-r lg:block hidden" />
                <div className="absolute top-0 left-3 right-3 h-0.5 bg-sky-500 rounded-b lg:hidden block" />
              </>
            )}
            <BookOpen size={20} />
          </button>

          {/* Sessions Icon */}
          <button
            onClick={() => handleTabClick('sessions')}
            className={cn(
              "relative p-3 rounded-xl transition-all duration-200 group flex items-center justify-center",
              activeTab === 'sessions' && sidebarOpen
                ? "text-sky-600 bg-sky-50/60"
                : "text-slate-500 hover:text-slate-800 hover:bg-slate-100/50"
            )}
            title="Chat Sessions"
          >
            {activeTab === 'sessions' && sidebarOpen && (
              <>
                <div className="absolute left-0 top-3 bottom-3 w-0.5 bg-sky-500 rounded-r lg:block hidden" />
                <div className="absolute top-0 left-3 right-3 h-0.5 bg-sky-500 rounded-b lg:hidden block" />
              </>
            )}
            <MessageSquare size={20} />
          </button>

          {/* Scheduler Icon */}
          <button
            onClick={() => handleTabClick('scheduler')}
            className={cn(
              "relative p-3 rounded-xl transition-all duration-200 group flex items-center justify-center",
              activeTab === 'scheduler' && sidebarOpen
                ? "text-sky-600 bg-sky-50/60"
                : "text-slate-500 hover:text-slate-800 hover:bg-slate-100/50"
            )}
            title="Task Scheduler"
          >
            {activeTab === 'scheduler' && sidebarOpen && (
              <>
                <div className="absolute left-0 top-3 bottom-3 w-0.5 bg-sky-500 rounded-r lg:block hidden" />
                <div className="absolute top-0 left-3 right-3 h-0.5 bg-sky-500 rounded-b lg:hidden block" />
              </>
            )}
            <CalendarRange size={20} />
          </button>

          {/* TODOs Icon */}
          <button
            onClick={() => handleTabClick('todos')}
            className={cn(
              "relative p-3 rounded-xl transition-all duration-200 group flex items-center justify-center",
              activeTab === 'todos' && sidebarOpen
                ? "text-sky-600 bg-sky-50/60"
                : "text-slate-500 hover:text-slate-800 hover:bg-slate-100/50"
            )}
            title="TODO List"
          >
            {activeTab === 'todos' && sidebarOpen && (
              <>
                <div className="absolute left-0 top-3 bottom-3 w-0.5 bg-sky-500 rounded-r lg:block hidden" />
                <div className="absolute top-0 left-3 right-3 h-0.5 bg-sky-500 rounded-b lg:hidden block" />
              </>
            )}
            <ListTodo size={20} />
          </button>
        </div>

        <div className="flex-col items-center gap-4 lg:flex hidden">
          <div className="p-1" title="Server Connected">
             <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
          </div>
        </div>
      </div>

      {/* Collapsible Panel */}
      <div className={cn(
        "bg-white flex flex-col transition-all duration-300 shrink-0 z-30 shadow-[1px_0_10px_rgba(0,0,0,0.01)] overflow-hidden",
        "w-full fixed inset-x-0 top-0 bottom-16 lg:relative lg:inset-auto lg:h-full lg:w-72 lg:border-r lg:border-slate-200/60 lg:shadow-none",
        !sidebarOpen 
          ? "translate-y-full opacity-0 pointer-events-none lg:translate-y-0 lg:opacity-100 lg:pointer-events-auto lg:w-0 lg:border-r-0" 
          : "translate-y-0 opacity-100 pointer-events-auto"
      )}>
        <div className="flex flex-col h-full w-full lg:w-72">
          {/* Header of the panel */}
          <div className="px-4 h-12 border-b border-slate-100 flex items-center justify-between bg-slate-50/50 shrink-0">
            <span className="font-display font-bold text-xs uppercase tracking-wider text-slate-500">
              {activeTab === 'files' && 'File Explorer'}
              {activeTab === 'wiki' && 'Wiki Vault'}
              {activeTab === 'sessions' && 'Recent Sessions'}
              {activeTab === 'scheduler' && 'Task Scheduler'}
              {activeTab === 'todos' && 'TODO list'}
            </span>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-hidden">
            {activeTab === 'files' && (
              <div className="h-full overflow-hidden bg-white">
                <FileExplorer onOpenFile={setActivePreviewPath} />
              </div>
            )}
            {activeTab === 'wiki' && (
              <div className="h-full overflow-hidden bg-white">
                <WikiExplorer onOpenFile={setActivePreviewPath} />
              </div>
            )}
            {activeTab === 'sessions' && (
              <div className="h-full overflow-hidden p-3 bg-white">
                <div className="font-display font-semibold text-[10px] text-slate-400 uppercase tracking-wider mb-3 px-1">
                  Active Chats
                </div>
                <div className="overflow-y-auto h-[calc(100vh-140px)] space-y-1.5 pr-1 scrollbar-thin">
                  {threads.length === 0 ? (
                    <div className="text-center text-xs text-slate-400 py-8 italic font-sans">
                      No active sessions yet.
                    </div>
                  ) : (
                    threads.map(thread => (
                      <div key={thread.id} 
                           onClick={() => setActiveThreadId(thread.id)}
                           className={cn(
                             "group p-2.5 rounded-lg border text-sm cursor-pointer hover:bg-slate-50 transition-all duration-200 shadow-sm/5 flex justify-between items-center",
                             activeThreadId === thread.id 
                               ? "border-sky-200 bg-sky-50/20" 
                               : "border-slate-100 bg-slate-50/30"
                           )}>
                         <div className="min-w-0 flex-1 pr-2">
                           <div className="font-medium text-slate-700 truncate group-hover:text-slate-900 transition-colors">
                             {thread.title}
                           </div>
                            <div className="text-[10px] text-slate-400 mt-1 flex gap-2 items-center font-mono font-medium">
                              {(() => {
                                const ts = thread.createdAt;
                                if (!ts || isNaN(ts)) return null;
                                return (
                                  <span title={new Date(ts).toLocaleString()}>
                                    {new Date(ts).toLocaleString(undefined, {
                                      month: 'short',
                                      day: 'numeric',
                                      hour: '2-digit',
                                      minute: '2-digit'
                                    })}
                                  </span>
                                );
                              })()}
                            </div>
                         </div>
                         <button
                           onClick={(e) => {
                             e.stopPropagation();
                             deleteThread(thread.id);
                           }}
                           className="opacity-0 group-hover:opacity-100 p-1 text-slate-400 hover:text-rose-500 hover:bg-rose-50 rounded-md transition-all shrink-0"
                           title="Delete Thread"
                         >
                           <Trash2 size={14} />
                         </button>
                      </div>
                    ))
                  )}
                </div>
              </div>
            )}
            {activeTab === 'scheduler' && (
              <SchedulerExplorer />
            )}
            {activeTab === 'todos' && (
              <TodoExplorer />
            )}
          </div>

          {/* <div className="p-3 bg-slate-50/50 border-t border-slate-100 flex justify-between items-center h-12 shrink-0">
              <span className="font-display font-semibold text-[10px] text-slate-400 uppercase tracking-wider">
                Server Connection
              </span>
              <ServerStatusIndicator />
          </div> */}
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
