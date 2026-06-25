import React, { useEffect, useState } from 'react';
import { api } from '../lib/api';
import { CheckSquare, Square, Trash2, Plus, Loader2, ListTodo } from 'lucide-react';

interface Todo {
  id: number;
  description: string;
  done: boolean;
}

export const TodoExplorer: React.FC = () => {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [loading, setLoading] = useState(true);
  const [newTodoDesc, setNewTodoDesc] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchTodos = async () => {
    try {
      const data = await api.listTodos();
      // Ensure we always have an array
      setTodos(data.todos || []);
    } catch (err: any) {
      console.error('Failed to list todos', err);
      setError('Failed to load TODOs');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchTodos();
  }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTodoDesc.trim()) return;

    setSubmitting(true);
    setError(null);

    try {
      await api.addTodo(newTodoDesc.trim());
      setNewTodoDesc('');
      await fetchTodos();
    } catch (err: any) {
      setError(err.message || 'Failed to add TODO');
    } finally {
      setSubmitting(false);
    }
  };

  const handleToggle = async (id: number) => {
    // Optimistic UI update
    setTodos(prev =>
      prev.map(todo => (todo.id === id ? { ...todo, done: !todo.done } : todo))
    );

    try {
      await api.toggleTodo(id);
    } catch (err: any) {
      console.error('Failed to toggle todo', err);
      // Rollback on error
      fetchTodos();
    }
  };

  const handleDelete = async (id: number) => {
    // Optimistic UI update
    setTodos(prev => prev.filter(todo => todo.id !== id));

    try {
      await api.deleteTodo(id);
    } catch (err: any) {
      console.error('Failed to delete todo', err);
      // Rollback on error
      fetchTodos();
    }
  };

  const pendingTodos = todos.filter(t => !t.done);
  const completedTodos = todos.filter(t => t.done);

  if (loading) {
    return (
      <div className="p-6 flex flex-col items-center justify-center h-full gap-2">
        <Loader2 className="animate-spin text-sky-500" size={24} />
        <span className="text-xs text-slate-400 font-sans">Loading TODOs...</span>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-white select-none">
      {/* Quick Add Form */}
      <form onSubmit={handleCreate} className="p-3 border-b border-slate-100 shrink-0">
        <div className="flex gap-2">
          <input
            type="text"
            value={newTodoDesc}
            onChange={(e) => setNewTodoDesc(e.target.value)}
            placeholder="Add new TODO..."
            disabled={submitting}
            className="flex-1 px-3 py-1.5 text-xs bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:ring-1 focus:ring-sky-500/50 focus:border-sky-500 transition-all disabled:opacity-60 placeholder:text-slate-400 font-sans"
          />
          <button
            type="submit"
            disabled={submitting || !newTodoDesc.trim()}
            className="p-1.5 bg-sky-500 hover:bg-sky-600 disabled:bg-slate-100 disabled:text-slate-400 text-white rounded-lg transition-colors flex items-center justify-center shrink-0"
            title="Add Todo"
          >
            {submitting ? (
              <Loader2 className="animate-spin" size={14} />
            ) : (
              <Plus size={14} />
            )}
          </button>
        </div>
        {error && (
          <div className="text-[10px] text-red-500 mt-1.5 px-1 font-sans">
            {error}
          </div>
        )}
      </form>

      {/* Todo Counts */}
      <div className="px-4 py-2 bg-slate-50/50 border-b border-slate-100 flex items-center justify-between text-[10px] font-medium text-slate-500 uppercase tracking-wider shrink-0 font-sans">
        <span>TODO List</span>
        <span className="text-[10px] font-mono lowercase text-slate-400">
          {pendingTodos.length} pending / {completedTodos.length} done
        </span>
      </div>

      {/* Scrollable Todo List */}
      <div className="flex-1 overflow-y-auto p-3 space-y-2.5 scrollbar-thin">
        {todos.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-slate-400 gap-2">
            <ListTodo size={28} className="text-slate-300 stroke-[1.5]" />
            <div className="text-xs italic font-sans">No TODOs found.</div>
            <p className="text-[10px] text-slate-400 text-center max-w-[160px] font-sans">
              Add one above to get started tracking your workspace tasks.
            </p>
          </div>
        ) : (
          <>
            {/* Pending List */}
            {pendingTodos.length > 0 && (
              <div className="space-y-1.5">
                {pendingTodos.map(todo => (
                  <div
                    key={todo.id}
                    className="group flex items-center justify-between p-2.5 rounded-lg border border-slate-100 bg-white hover:bg-slate-50/50 hover:border-slate-200/80 transition-all duration-200 shadow-sm/5"
                  >
                    <div className="flex items-center gap-2.5 flex-1 min-w-0">
                      <button
                        onClick={() => handleToggle(todo.id)}
                        className="text-slate-400 hover:text-sky-500 transition-colors shrink-0"
                        title="Mark Complete"
                      >
                        <Square size={15} />
                      </button>
                      <span className="text-xs text-slate-700 font-sans truncate pr-1">
                        {todo.description}
                      </span>
                    </div>
                    <button
                      onClick={() => handleDelete(todo.id)}
                      className="opacity-0 group-hover:opacity-100 p-1 text-slate-400 hover:text-red-500 hover:bg-red-50/85 rounded transition-all shrink-0"
                      title="Delete Todo"
                    >
                      <Trash2 size={13} />
                    </button>
                  </div>
                ))}
              </div>
            )}

            {/* Separator if both exist */}
            {pendingTodos.length > 0 && completedTodos.length > 0 && (
              <div className="border-t border-slate-100/80 my-2" />
            )}

            {/* Completed List */}
            {completedTodos.length > 0 && (
              <div className="space-y-1.5 opacity-65">
                {completedTodos.map(todo => (
                  <div
                    key={todo.id}
                    className="group flex items-center justify-between p-2.5 rounded-lg border border-slate-100 bg-slate-50/20 hover:bg-slate-50/60 transition-all duration-200"
                  >
                    <div className="flex items-center gap-2.5 flex-1 min-w-0">
                      <button
                        onClick={() => handleToggle(todo.id)}
                        className="text-emerald-500 hover:text-slate-400 transition-colors shrink-0"
                        title="Mark Incomplete"
                      >
                        <CheckSquare size={15} />
                      </button>
                      <span className="text-xs text-slate-500 line-through font-sans truncate pr-1">
                        {todo.description}
                      </span>
                    </div>
                    <button
                      onClick={() => handleDelete(todo.id)}
                      className="opacity-0 group-hover:opacity-100 p-1 text-slate-400 hover:text-red-500 hover:bg-red-50/85 rounded transition-all shrink-0"
                      title="Delete Todo"
                    >
                      <Trash2 size={13} />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};
