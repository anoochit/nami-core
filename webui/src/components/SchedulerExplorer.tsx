import React, { useEffect, useState, useRef } from 'react';
import { api } from '../lib/api';
import { Calendar, Clock, Loader2, Trash2, Plus, X, ToggleLeft, ToggleRight, Check, Terminal as TerminalIcon, RefreshCw } from 'lucide-react';

interface ScheduledTask {
  id: string;
  goal: string;
  cron_expr: string;
  last_run: string | null;
  is_active: boolean;
}

interface LogMessage {
  llm_response: string;
  author: string;
  timestamp: string;
}

export const SchedulerExplorer: React.FC = () => {
  const [tasks, setTasks] = useState<ScheduledTask[]>([]);
  const [loading, setLoading] = useState(true);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [goal, setGoal] = useState('');
  const [cronExpr, setCronExpr] = useState('0 0 * * * *');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  // Active Tab View: 'schedules' or 'logs'
  const [activeTab, setActiveTab] = useState<'schedules' | 'logs'>('schedules');

  // Terminal Log State
  const [logs, setLogs] = useState<LogMessage[]>([]);
  const [loadingLogs, setLoadingLogs] = useState(false);
  const [autoPoll, setAutoPoll] = useState(true);
  
  const terminalEndRef = useRef<HTMLDivElement>(null);

  const presets = [
    { label: 'Every minute', value: '0 * * * * *' },
    { label: 'Every 5 minutes', value: '0 */5 * * * *' },
    { label: 'Hourly', value: '0 0 * * * *' },
    { label: 'Daily at midnight', value: '0 0 0 * * *' },
  ];

  const fetchTasks = async () => {
    setLoading(true);
    try {
      const data = await api.listSchedulerTasks();
      setTasks(data);
    } catch (err) {
      console.error('Failed to list scheduler tasks', err);
    } finally {
      setLoading(false);
    }
  };

  const fetchLogs = async (silent = false) => {
    if (!silent) setLoadingLogs(true);
    try {
      const messages = await api.getSessionMessages('background_tasks');
      setLogs(messages);
    } catch (err) {
      console.error('Failed to fetch background execution logs', err);
    } finally {
      if (!silent) setLoadingLogs(false);
    }
  };

  useEffect(() => {
    fetchTasks();
    fetchLogs();
  }, []);

  // Set up real-time polling for logs when logs tab is active
  useEffect(() => {
    if (!autoPoll || activeTab !== 'logs') return;

    const interval = setInterval(() => {
      fetchLogs(true);
    }, 3000);

    return () => clearInterval(interval);
  }, [autoPoll, activeTab]);

  // Scroll terminal to bottom on new logs
  useEffect(() => {
    if (activeTab === 'logs') {
      terminalEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, activeTab]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!goal.trim() || !cronExpr.trim()) return;

    setSubmitting(true);
    setError(null);
    setSuccess(false);

    try {
      await api.addSchedulerTask(goal, cronExpr);
      setSuccess(true);
      setGoal('');
      setTimeout(() => {
        setIsFormOpen(false);
        setSuccess(false);
      }, 1500);
      fetchTasks();
    } catch (err: any) {
      setError(err.message || 'Failed to schedule task');
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this scheduled task?')) return;
    try {
      await api.deleteSchedulerTask(id);
      fetchTasks();
    } catch (err: any) {
      alert(err.message || 'Failed to delete task');
    }
  };

  const handleToggle = async (id: string) => {
    try {
      await api.toggleSchedulerTask(id);
      setTasks(prev =>
        prev.map(t => (t.id === id ? { ...t, is_active: !t.is_active } : t))
      );
    } catch (err: any) {
      alert(err.message || 'Failed to toggle task');
    }
  };

  if (loading) {
    return (
      <div className="p-8 flex flex-col items-center justify-center gap-3">
        <Loader2 className="animate-spin text-sky-500" size={24} />
        <span className="text-xs text-slate-400 font-sans">Loading schedule...</span>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden bg-white">
      {/* Header and Sleek Tab Navigation Menu */}
      <div className="border-b border-slate-100 bg-slate-50/40 shrink-0 flex flex-col">
        <div className="p-3 pb-2 flex justify-between items-center">
          <span className="text-xs font-bold text-slate-700 tracking-wider">
            SCHEDULER EXPLORER
          </span>
          {activeTab === 'schedules' && (
            <button
              onClick={() => setIsFormOpen(!isFormOpen)}
              className="flex items-center gap-1.5 px-2.5 py-1.5 text-[11px] font-semibold bg-sky-500 hover:bg-sky-600 text-white rounded-md shadow-sm transition-all duration-200"
            >
              {isFormOpen ? <X size={12} /> : <Plus size={12} />}
              <span>{isFormOpen ? 'Close' : 'Schedule'}</span>
            </button>
          )}
        </div>

        {/* Navigation Tabs */}
        <div className="flex px-3 border-t border-slate-100 bg-white">
          <button
            onClick={() => setActiveTab('schedules')}
            className={`flex items-center gap-2 py-2.5 px-3 text-xs font-semibold border-b-2 transition-all duration-200 ${
              activeTab === 'schedules'
                ? 'border-sky-500 text-sky-600'
                : 'border-transparent text-slate-400 hover:text-slate-600'
            }`}
          >
            <Clock size={13} />
            <span>Schedules ({tasks.length})</span>
          </button>
          <button
            onClick={() => setActiveTab('logs')}
            className={`flex items-center gap-2 py-2.5 px-3 text-xs font-semibold border-b-2 transition-all duration-200 relative ${
              activeTab === 'logs'
                ? 'border-emerald-500 text-emerald-600'
                : 'border-transparent text-slate-400 hover:text-slate-600'
            }`}
          >
            <TerminalIcon size={13} />
            <span>Execution Logs</span>
            {autoPoll && (
              <span className="absolute right-1 top-2.5 w-1.5 h-1.5 bg-emerald-500 rounded-full animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.6)]" />
            )}
          </button>
        </div>
      </div>

      {activeTab === 'schedules' ? (
        /* Schedules Tab: Lists all repeating tasks & the creation form */
        <div className="flex-1 overflow-y-auto p-3 space-y-3.5 scrollbar-thin">
          {/* New Task Form */}
          {isFormOpen && (
            <form onSubmit={handleCreate} className="p-3.5 rounded-xl border border-slate-200 bg-slate-50/50 shadow-sm/5 space-y-3.5 animate-in slide-in-from-top duration-200">
              <div>
                <label className="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1.5">
                  Task Goal
                </label>
                <textarea
                  value={goal}
                  onChange={e => setGoal(e.target.value)}
                  placeholder="What objective should the agent achieve periodically?"
                  className="w-full text-xs p-2 rounded-lg border border-slate-200 focus:outline-none focus:ring-1 focus:ring-sky-400 bg-white placeholder-slate-400 resize-none h-16 font-sans"
                  required
                />
              </div>

              <div className="grid grid-cols-1 gap-3">
                <div>
                  <label className="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1.5">
                    Presets Helper
                  </label>
                  <div className="flex flex-wrap gap-1.5">
                    {presets.map(p => (
                      <button
                        key={p.value}
                        type="button"
                        onClick={() => setCronExpr(p.value)}
                        className={`px-2 py-1 rounded text-[10px] font-medium transition-colors ${
                          cronExpr === p.value
                            ? 'bg-sky-100 text-sky-700 border border-sky-200'
                            : 'bg-white text-slate-500 border border-slate-200 hover:bg-slate-50'
                        }`}
                      >
                        {p.label}
                      </button>
                    ))}
                  </div>
                </div>

                <div>
                  <label className="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1.5 flex items-center justify-between">
                    <span>Cron Expression</span>
                    <span className="text-[9px] lowercase font-normal text-slate-400">supports 6 fields</span>
                  </label>
                  <input
                    type="text"
                    value={cronExpr}
                    onChange={e => setCronExpr(e.target.value)}
                    placeholder="0 0 * * * *"
                    className="w-full text-xs px-2.5 py-2 rounded-lg border border-slate-200 focus:outline-none focus:ring-1 focus:ring-sky-400 bg-white font-mono"
                    required
                  />
                </div>
              </div>

              {error && (
                <div className="text-[10px] font-medium text-red-500 bg-red-50 border border-red-100 p-2 rounded-lg">
                  {error}
                </div>
              )}

              {success && (
                <div className="text-[10px] font-medium text-emerald-600 bg-emerald-50 border border-emerald-100 p-2 rounded-lg flex items-center gap-1.5">
                  <Check size={12} />
                  <span>Task scheduled successfully!</span>
                </div>
              )}

              <button
                type="submit"
                disabled={submitting}
                className="w-full flex items-center justify-center gap-1.5 py-2 bg-slate-800 hover:bg-slate-900 text-white font-medium rounded-lg text-xs transition-colors"
              >
                {submitting ? (
                  <Loader2 className="animate-spin text-white" size={14} />
                ) : (
                  <>
                    <Clock size={14} />
                    <span>Create Repeating Schedule</span>
                  </>
                )}
              </button>
            </form>
          )}

          {/* Tasks List */}
          {tasks.length === 0 ? (
            <div className="text-center text-xs text-slate-400 py-12 italic font-sans flex flex-col items-center justify-center gap-2">
              <Clock className="text-slate-300" size={24} />
              <span>No background tasks configured yet.</span>
            </div>
          ) : (
            <div className="space-y-2.5">
              {tasks.map(task => (
                <div
                  key={task.id}
                  className={`p-3 rounded-xl border transition-all duration-200 shadow-sm/5 flex flex-col justify-between gap-2.5 relative group ${
                    task.is_active
                      ? 'bg-white border-slate-200/80 hover:border-slate-300'
                      : 'bg-slate-50/50 border-slate-150 text-slate-400'
                  }`}
                >
                  <div className="flex justify-between items-start">
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleToggle(task.id)}
                        className={`transition-colors outline-none focus:ring-0 ${
                          task.is_active ? 'text-emerald-500 hover:text-emerald-600' : 'text-slate-300 hover:text-slate-400'
                        }`}
                        title={task.is_active ? 'Deactivate schedule' : 'Activate schedule'}
                      >
                        {task.is_active ? <ToggleRight size={22} /> : <ToggleLeft size={22} />}
                      </button>
                      <span className={`text-[10px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded ${
                        task.is_active ? 'bg-emerald-50 text-emerald-600' : 'bg-slate-100 text-slate-500'
                      }`}>
                        {task.is_active ? 'Active' : 'Inactive'}
                      </span>
                    </div>

                    <button
                      onClick={() => handleDelete(task.id)}
                      className="p-1 text-slate-400 hover:text-red-500 rounded transition-colors"
                      title="Delete schedule"
                    >
                      <Trash2 size={13} />
                    </button>
                  </div>

                  <div className="text-xs leading-relaxed font-medium font-sans">
                    {task.goal}
                  </div>

                  <div className="border-t border-slate-100/80 pt-2 flex flex-col gap-1 text-[10px] text-slate-400">
                    <div className="flex items-center gap-1.5">
                      <Clock size={11} className="shrink-0 text-slate-400/80" />
                      <span className="font-mono text-slate-600">{task.cron_expr}</span>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <Calendar size={11} className="shrink-0 text-slate-400/80" />
                      <span>
                        Last Run:{' '}
                        {task.last_run
                          ? new Date(task.last_run).toLocaleString()
                          : 'Never run'}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      ) : (
        /* Logs Tab: Centered full-stage terminal stream console */
        <div className="flex-1 bg-slate-950 flex flex-col overflow-hidden relative">
          {/* Terminal Actions Bar inside Logs view */}
          <div className="px-4 py-2 bg-slate-900 border-b border-slate-800 flex items-center justify-between text-slate-300 shrink-0">
            <div className="flex items-center gap-2">
              <span className="font-mono text-[10px] text-slate-400 font-bold uppercase tracking-wider">Live stream feed</span>
              <div className="w-1.5 h-1.5 bg-emerald-500 rounded-full animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.6)]" />
            </div>
            <div className="flex items-center gap-2.5">
              <button 
                onClick={() => setAutoPoll(!autoPoll)}
                className={`text-[9px] font-bold px-1.5 py-0.5 rounded uppercase tracking-wider transition-all duration-200 border ${
                  autoPoll 
                    ? 'bg-emerald-950/40 text-emerald-400 border-emerald-800/60' 
                    : 'bg-slate-950 text-slate-400 border-slate-800'
                }`}
                title="Toggle Live Stream Polling"
              >
                {autoPoll ? 'live' : 'paused'}
              </button>
              <button 
                onClick={() => fetchLogs()}
                className="p-1 text-slate-400 hover:text-slate-200 rounded transition-colors"
                title="Force Refresh Logs"
              >
                <RefreshCw size={11} className={loadingLogs ? 'animate-spin text-emerald-400' : ''} />
              </button>
            </div>
          </div>

          {/* Centered Scrollable Terminal Console */}
          <div className="flex-1 p-4 font-mono text-[11px] text-slate-300 overflow-y-auto leading-relaxed scrollbar-thin select-text">
            {/* Terminal System Header */}
            {logs.length === 0 ? (
              <div className="text-slate-500 italic py-6 text-center font-sans">
                No logs streamed yet. Background tasks will print execution streams here when triggered.
              </div>
            ) : (
              <div className="space-y-3">
                {logs.map((log, i) => {
                  const isUser = log.author === 'user' || log.author === 'system';
                  const timestampStr = log.timestamp ? new Date(log.timestamp).toLocaleTimeString() : '';
                  return (
                    <div key={i} className="animate-in fade-in duration-200">
                      <div className="flex items-center gap-2 text-slate-500 mb-1">
                        <span className="text-[10px] bg-slate-900 px-1.5 py-0.5 rounded border border-slate-800/80 font-mono">
                          {timestampStr}
                        </span>
                        <span className={`text-[10px] font-bold font-mono ${isUser ? 'text-blue-400' : 'text-emerald-400'}`}>
                          [{log.author.toUpperCase()}]
                        </span>
                      </div>
                      <div className="pl-3 border-l-2 border-slate-800 text-slate-200 whitespace-pre-wrap selection:bg-emerald-500/20">
                        {log.llm_response}
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
            <div ref={terminalEndRef} />
          </div>
        </div>
      )}
    </div>
  );
};
