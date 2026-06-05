import React, { useEffect, useState, useMemo } from 'react';
import { api } from '../lib/api';
import { FileText, Folder, Loader2, ArrowLeft } from 'lucide-react';

interface WikiExplorerProps {
  onOpenFile: (path: string) => void;
}

export const WikiExplorer: React.FC<WikiExplorerProps> = ({ onOpenFile }) => {
  const [pages, setPages] = useState<string[]>([]);
  const [currentPath, setCurrentPath] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchPages = async () => {
      setLoading(true);
      try {
        const data = await api.listWikiPages();
        setPages(data.pages);
      } catch (err) {
        console.error("Failed to list wiki pages", err);
      } finally {
        setLoading(false);
      }
    };
    fetchPages();
  }, []);

  const entries = useMemo(() => {
    const entriesMap = new Map<string, { name: string; type: 'file' | 'folder'; fullPath: string }>();

    pages.forEach(page => {
      // Check if page is inside currentPath
      const prefix = currentPath ? `${currentPath}/` : "";
      if (!page.startsWith(prefix)) return;

      const relativePart = page.substring(prefix.length);
      const segments = relativePart.split('/');
      const firstSegment = segments[0];
      if (!firstSegment) return;

      const isFolder = segments.length > 1;
      const key = `${isFolder ? 'folder' : 'file'}:${firstSegment}`;

      if (!entriesMap.has(key)) {
        entriesMap.set(key, {
          name: firstSegment,
          type: isFolder ? 'folder' : 'file',
          fullPath: currentPath ? `${currentPath}/${firstSegment}` : firstSegment,
        });
      }
    });

    const sorted = Array.from(entriesMap.values());
    sorted.sort((a, b) => {
      if (a.type !== b.type) {
        return a.type === 'folder' ? -1 : 1; // Folder comes before file
      }
      return a.name.localeCompare(b.name);
    });
    return sorted;
  }, [pages, currentPath]);

  const handleFolderClick = (folderName: string) => {
    setCurrentPath(prev => prev ? `${prev}/${folderName}` : folderName);
  };

  const handleBack = (e: React.MouseEvent) => {
    e.stopPropagation();
    const segments = currentPath.split('/');
    segments.pop();
    setCurrentPath(segments.join('/'));
  };

  if (loading) {
    return (
      <div className="p-4 flex justify-center">
        <Loader2 className="animate-spin text-slate-500" />
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      {currentPath && (
        <div 
          onClick={handleBack} 
          className="flex items-center gap-2 p-2.5 cursor-pointer hover:bg-slate-50 border-b border-slate-100 text-xs font-medium text-slate-500 transition-colors"
        >
          <ArrowLeft size={14} />
          <span>Back to parent</span>
        </div>
      )}
      
      {entries.length === 0 ? (
        <div className="text-center text-xs text-slate-400 py-8 italic font-sans">
          No wiki pages found.
        </div>
      ) : (
        entries.map((entry) => (
          <div 
            key={entry.fullPath} 
            onClick={() => {
              if (entry.type === 'file') {
                // Prepend 'wiki/' so the file preview component knows it is a wiki file
                onOpenFile(`wiki/${entry.fullPath}.md`);
              } else {
                handleFolderClick(entry.name);
              }
            }}
            className="flex items-center gap-2.5 px-3 py-2.5 cursor-pointer hover:bg-slate-50/50 rounded-lg text-xs text-slate-600 hover:text-slate-900 transition-all duration-150"
          >
            {entry.type === 'folder' ? (
              <Folder size={14} className="text-indigo-400/80 shrink-0" />
            ) : (
              <FileText size={14} className="text-slate-400 shrink-0" />
            )}
            <span className="truncate font-medium">{entry.name}</span>
          </div>
        ))
      )}
    </div>
  );
};
