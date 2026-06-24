import React, { useEffect, useState } from 'react';
import { api } from '../lib/api';
import { Calendar, Clock, Loader2, Trash2, Plus, X, ToggleLeft, ToggleRight, Check } from 'lucide-react';

interface ScheduledTask {
  id: string;
  goal: string;
  cron_expr: string;
  last_run: string | null;
  is_active: boolean;
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

  useEffect(() => {
    fetchTasks();
  }, []);

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
      {/* Header action */}
      <div className="p-3 border-b border-slate-100 bg-slate-50/40 flex justify-between items-center shrink-0">
        <span className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider px-1">
          Active Schedules ({tasks.length})
        </span>
        <button
          onClick={() => setIsFormOpen(!isFormOpen)}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-[11px] font-semibold bg-sky-500 hover:bg-sky-600 text-white rounded-md shadow-sm transition-all duration-200"
        >
          {isFormOpen ? <X size={12} /> : <Plus size={12} />}
          <span>{isFormOpen ? 'Close' : 'Schedule Task'}</span>
        </button>
      </div>

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
              - presets
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
                {/* Active Switch & Action */}
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

                {/* Goal description */}
                <div className="text-xs leading-relaxed font-medium font-sans">
                  {task.goal}
                </div>

                {/* Schedule Details */}
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
    </div>
  );
};
