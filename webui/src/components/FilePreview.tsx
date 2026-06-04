import React, { useState, useEffect, useRef } from 'react';
import { X, FileText, Loader2, AlertCircle, Download, Copy, Check } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { api, getHeaders } from '../lib/api';

const parseFrontmatter = (content: string) => {
    const match = content.match(/^---\r?\n([\s\S]+?)\r?\n---\r?\n([\s\S]*)$/);
    if (!match) return { data: {} as Record<string, string>, content };
    
    const [_, yaml, markdown] = match;
    const data: Record<string, string> = {};
    yaml.split('\n').forEach(line => {
        const [key, ...value] = line.split(':');
        if (key && value) data[key.trim()] = value.join(':').trim();
    });
    return { data, content: markdown };
};

interface FilePreviewProps {
  path: string | null;
  onClose: () => void;
  isWiki?: boolean;
}

export const FilePreview: React.FC<FilePreviewProps> = ({ path, onClose, isWiki = false }) => {
  const [content, setContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [mediaUrl, setMediaUrl] = useState<string | null>(null);
  const [marpHtml, setMarpHtml] = useState<string>('');
  const [copied, setCopied] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const handleDownload = async () => {
    if (!path) return;
    try {
        const response = await fetch(`/api/workspace/read-binary/${path}`, { headers: getHeaders() });
        const blob = await response.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = path.split('/').pop() || 'file';
        a.click();
        URL.revokeObjectURL(url);
    } catch (e) {
        console.error("Download failed", e);
    }
  };

  const handleCopy = () => {
      if (content) {
          navigator.clipboard.writeText(content);
          setCopied(true);
          setTimeout(() => setCopied(false), 2000);
      }
  };

  useEffect(() => {
    if (!path) {
      setContent(null);
      setMediaUrl(null);
      setMarpHtml('');
      return;
    }

    const fetchContent = async () => {
      setLoading(true);
      setError(null);
      setMarpHtml('');
      
      if (mediaUrl) {
        URL.revokeObjectURL(mediaUrl);
        setMediaUrl(null);
      }

      try {
        const ext = path.split('.').pop()?.toLowerCase() || '';
        const imageExtensions = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'];
        const videoExtensions = ['mp4', 'webm'];
        const audioExtensions = ['mp3', 'wav', 'ogg', 'm4a'];
        
        const isImage = imageExtensions.includes(ext);
        const isVideo = videoExtensions.includes(ext);
        const isAudio = audioExtensions.includes(ext);
        const isPdf = ext === 'pdf';

        if (isWiki) {
           const response = await fetch(`/api/wiki/pages/${path}`, {
               headers: getHeaders()
           });
           if (!response.ok) throw new Error('Failed to load wiki page');
           const data = await response.json();
           setContent(data.content);
        } else if (isPdf) {
           const response = await fetch(`/api/workspace/read-binary/${path}`, {
               headers: getHeaders()
           });
           if (!response.ok) throw new Error('Failed to load PDF');
           const blob = await response.blob();
           await renderPdf(blob);
           setContent("");
        } else if (isImage || isVideo || isAudio) {
           const response = await fetch(`/api/workspace/read-binary/${path}`, {
               headers: getHeaders()
           });
           if (!response.ok) throw new Error('Failed to load media');
           const blob = await response.blob();
           const url = URL.createObjectURL(blob);
           setMediaUrl(url);
           setContent("");
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

    const renderPdf = async (blob: Blob) => {
        const pdfjsLib = await import('pdfjs-dist');
        const pdfWorker = await import('pdfjs-dist/build/pdf.worker.min?url');
        pdfjsLib.GlobalWorkerOptions.workerSrc = pdfWorker.default;

        const arrayBuffer = await blob.arrayBuffer();
        const loadingTask = pdfjsLib.getDocument({ data: arrayBuffer });
        const pdf = await loadingTask.promise;
        const page = await pdf.getPage(1);
        const viewport = page.getViewport({ scale: 1.5 });
        const canvas = canvasRef.current;
        if (!canvas) return;
        const context = canvas.getContext('2d');
        if (!context) return;
        canvas.height = viewport.height;
        canvas.width = viewport.width;
        await page.render({ canvasContext: context, viewport, canvas }).promise;
    };

    fetchContent();

    return () => {
        if (mediaUrl) URL.revokeObjectURL(mediaUrl);
    };
  }, [path, isWiki]);

  useEffect(() => {
    const processMarp = async () => {
        if (!content) return;
        const ext = path?.split('.').pop()?.toLowerCase() || '';
        const isMarkdown = ext === 'md';
        const parsed = isMarkdown ? parseFrontmatter(content) : { data: {} as Record<string, string>, content };
        const isMarp = isMarkdown && parsed.data['marp'] === 'true';

        if (isMarp) {
            const { Marp } = await import('@marp-team/marp-core');
            const marp = new Marp();
            const { html, css } = marp.render(content);
            setMarpHtml(`<style>${css}</style>${html}`);
        }
    };
    processMarp();
  }, [content, path]);

  if (!path) return null;

  const ext = path.split('.').pop()?.toLowerCase() || '';
  const isMarkdown = ext === 'md';
  const parsed = isMarkdown && content ? parseFrontmatter(content) : { data: {} as Record<string, string>, content: content || '' };
  const isMarp = isMarkdown && parsed.data['marp'] === 'true';

  const isImage = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'].includes(ext);
  const isVideo = ['mp4', 'webm'].includes(ext);
  const isAudio = ['mp3', 'wav', 'ogg', 'm4a'].includes(ext);
  const isPdf = ext === 'pdf';
  const isHtml = ['html', 'htm'].includes(ext);
  const isCode = ['ts', 'tsx', 'js', 'jsx', 'rs', 'py', 'json', 'css'].includes(ext);

  return (
    <div className="flex flex-col h-full bg-white border-l shadow-2xl z-10">
      <div className="p-3 border-b flex justify-between items-center bg-gray-50">
        <div className="flex items-center gap-2 overflow-hidden">
          <FileText size={18} className="text-blue-500 shrink-0" />
          <span className="font-medium text-sm truncate" title={path}>{path}</span>
        </div>
        <div className="flex gap-1">
          {content !== null && (
             <button onClick={handleCopy} className="p-1 hover:bg-gray-200 rounded-md transition-colors" title="Copy to clipboard">
                {copied ? <Check size={18} className="text-green-600" /> : <Copy size={18} />}
             </button>
          )}
          <button onClick={handleDownload} className="p-1 hover:bg-gray-200 rounded-md transition-colors" title="Download file">
             <Download size={18} />
          </button>
          <button onClick={onClose} className="p-1 hover:bg-gray-200 rounded-md transition-colors"><X size={20} /></button>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {loading ? (
          <div className="h-full flex flex-col items-center justify-center text-gray-400 gap-2">
            <Loader2 className="animate-spin" size={32} />
            <p className="text-sm">Loading file preview...</p>
          </div>
        ) : error ? (
          <div className="h-full flex flex-col items-center justify-center text-red-500 gap-3 p-6 text-center">
            <AlertCircle size={40} />
            <div>
              <p className="font-bold">Error Loading File</p>
              <p className="text-sm opacity-80">{error}</p>
            </div>
          </div>
        ) : (
          <div className="prose prose-sm max-w-none h-full">
            {isPdf ? (
              <canvas ref={canvasRef} className="w-full h-auto" />
            ) : isImage && mediaUrl ? (
              <img src={mediaUrl} alt={path} className="max-w-full" />
            ) : isVideo && mediaUrl ? (
              <video controls src={mediaUrl} className="max-w-full" />
            ) : isAudio && mediaUrl ? (
              <audio controls src={mediaUrl} className="w-full" />
            ) : isHtml ? (
              <iframe srcDoc={content || ''} title="HTML Preview" className="w-full h-full border-none" />
            ) : isMarp ? (
                <div dangerouslySetInnerHTML={{ __html: marpHtml }} />
            ) : isCode ? (
                <SyntaxHighlighter language={ext} style={vscDarkPlus} customStyle={{ margin: 0, padding: '1rem', borderRadius: '0.5rem' }}>
                    {content || ''}
                </SyntaxHighlighter>
            ) : (
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{parsed.content}</ReactMarkdown>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
