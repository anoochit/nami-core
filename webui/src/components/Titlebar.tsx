import React, { useEffect, useState } from 'react';
import { Minus, Square, X } from 'lucide-react';

export const Titlebar: React.FC = () => {
  const [isTauri, setIsTauri] = useState(false);

  useEffect(() => {
    if (typeof window !== 'undefined' && (window as any).__TAURI__) {
      setIsTauri(true);
    }
  }, []);

  if (!isTauri) return null;

  const handleMinimize = () => {
    (window as any).__TAURI__.core.invoke('minimize_window').catch(console.error);
  };

  const handleMaximize = () => {
    (window as any).__TAURI__.core.invoke('maximize_window').catch(console.error);
  };

  const handleClose = () => {
    (window as any).__TAURI__.core.invoke('close_window').catch(console.error);
  };

  return (
    <div 
      data-tauri-drag-region 
      className="h-10 bg-slate-900 text-slate-100 flex items-center justify-between px-4 select-none border-b border-slate-800 shrink-0 z-50 cursor-default"
    >
      <div data-tauri-drag-region className="flex items-center gap-2">
        <span data-tauri-drag-region className="w-2.5 h-2.5 rounded-full bg-indigo-500 animate-pulse" />
        <span data-tauri-drag-region className="text-xs font-semibold font-display tracking-wider uppercase opacity-90">Nami AI Desktop</span>
      </div>
      
      <div className="flex items-center">
        <button 
          onClick={handleMinimize}
          className="p-1.5 hover:bg-slate-800 text-slate-400 hover:text-slate-100 transition-colors duration-150 rounded"
          title="Minimize"
        >
          <Minus size={14} />
        </button>
        <button 
          onClick={handleMaximize}
          className="p-1.5 hover:bg-slate-800 text-slate-400 hover:text-slate-100 transition-colors duration-150 rounded"
          title="Maximize"
        >
          <Square size={12} />
        </button>
        <button 
          onClick={handleClose}
          className="p-1.5 hover:bg-rose-600/90 text-slate-400 hover:text-white transition-colors duration-150 rounded ml-1"
          title="Close"
        >
          <X size={14} />
        </button>
      </div>
    </div>
  );
};
