import React, { useState, useEffect } from 'react';
import { X, FileText, Loader2, AlertCircle } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { api } from '../lib/api';

interface PreviewPaneProps {
  path: string | null;
  onClose: () => void;
  isWiki?: boolean;
}

export const PreviewPane: React.FC<PreviewPaneProps> = ({ path, onClose, isWiki = false }) => {
  const [content, setContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!path) {
      setContent(null);
      return;
    }

    const fetchContent = async () => {
      setLoading(true);
      setError(null);
      try {
        if (isWiki) {
           const response = await fetch(`/api/wiki/pages/${path}`);
           if (!response.ok) throw new Error('Failed to load wiki page');
           const data = await response.json();
           setContent(data.content);
        } else {
           const data = await api.readWorkspaceFile(path);
           setContent(data.content);
        }
      } catch (err: any) {
        setError(err.message || 'Failed to load content');
      } finally {
        setLoading(false);
      }
    };

    fetchContent();
  }, [path, isWiki]);

  if (!path) return null;

  const ext = path.split('.').pop()?.toLowerCase() || '';

  const codeExtensions = ['rs', 'ts', 'tsx', 'js', 'jsx', 'json', 'toml', 'yml', 'yaml', 'txt', 'css', 'html', 'htm', 'py', 'java', 'cpp', 'h', 'cs', 'xml', 'csv', 'sh', 'ps1', 'bat', 'sql'];
  const imageExtensions = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'];
  const videoExtensions = ['mp4', 'webm'];
  const audioExtensions = ['mp3', 'wav', 'ogg', 'm4a'];

  const isMarkdown = ext === 'md';
  const isCode = codeExtensions.includes(ext);
  const isImage = imageExtensions.includes(ext);
  const isVideo = videoExtensions.includes(ext);
  const isAudio = audioExtensions.includes(ext);

  const formattedContent = isCode 
    ? `\`\`\`${ext}\n${content}\n\`\`\``
    : content;

  return (
    <div className="flex flex-col h-full bg-white border-l shadow-2xl z-10">
      <div className="p-3 border-b flex justify-between items-center bg-gray-50">
        <div className="flex items-center gap-2 overflow-hidden">
          <FileText size={18} className="text-blue-500 shrink-0" />
          <span className="font-medium text-sm truncate" title={path}>{path}</span>
        </div>
        <button 
          onClick={onClose}
          className="p-1 hover:bg-gray-200 rounded-md transition-colors"
        >
          <X size={20} />
        </button>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {loading ? (
          <div className="h-full flex flex-col items-center justify-center text-gray-400 gap-2">
            <Loader2 className="animate-spin" size={32} />
            <p className="text-sm">Loading artifact...</p>
          </div>
        ) : error ? (
          <div className="h-full flex flex-col items-center justify-center text-red-500 gap-3 p-6 text-center">
            <AlertCircle size={40} />
            <div>
              <p className="font-bold">Error Loading File</p>
              <p className="text-sm opacity-80">{error}</p>
            </div>
          </div>
        ) : content !== null ? (
          <div className="prose prose-sm max-w-none prose-pre:bg-gray-900 prose-pre:text-gray-100">
            {isImage ? (
              <img src={`/api/workspace/read/${path}`} alt={path} className="max-w-full" />
            ) : isVideo ? (
              <video controls src={`/api/workspace/read/${path}`} className="max-w-full" />
            ) : isAudio ? (
              <audio controls src={`/api/workspace/read/${path}`} className="w-full" />
            ) : (
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {formattedContent || ''}
              </ReactMarkdown>
            )}
          </div>
        ) : (
          <div className="h-full flex items-center justify-center text-gray-400 italic">
            Select a file to preview
          </div>
        )}
      </div>
    </div>
  );
};
