import React, { useEffect, useState } from 'react';
import { api } from '../lib/api';
import { File, Folder, Loader2, ArrowLeft } from 'lucide-react';

interface FileExplorerProps {
  onOpenFile: (path: string) => void;
}

export const FileExplorer: React.FC<FileExplorerProps> = ({ onOpenFile }) => {
  const [files, setFiles] = useState<Array<{ name: string, type: string }>>([]);
  const [currentPath, setCurrentPath] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchFiles = async () => {
      setLoading(true);
      try {
        const data = await api.listWorkspaceFiles(currentPath);
        setFiles(data.entries);
      } catch (err) {
        console.error("Failed to list files", err);
      } finally {
        setLoading(false);
      }
    };
    fetchFiles();
  }, [currentPath]);

  const handleFolderClick = (folderName: string) => {
    setCurrentPath(prev => prev ? `${prev}/${folderName}` : folderName);
  };

  const handleBack = () => {
    setCurrentPath(prev => {
        const parts = prev.split('/');
        parts.pop();
        return parts.join('/');
    });
  };

  if (loading) return <div className="p-4 flex justify-center"><Loader2 className="animate-spin text-gray-500" /></div>;

  return (
    <div className="flex flex-col h-full overflow-y-auto">
        {currentPath && (
            <div onClick={handleBack} className="flex items-center gap-2 p-2 cursor-pointer hover:bg-gray-100 rounded text-sm text-gray-600 border-b">
                <ArrowLeft size={16} />
                <span>..</span>
            </div>
        )}
        {files.map((file) => (
            <div 
                key={file.name} 
                onClick={() => file.type === 'file' ? onOpenFile(currentPath ? `${currentPath}/${file.name}` : file.name) : handleFolderClick(file.name)}
                className="flex items-center gap-2 p-2 cursor-pointer hover:bg-gray-100 rounded text-sm"
            >
                {file.type === 'folder' ? <Folder size={16} className="text-blue-500" /> : <File size={16} className="text-gray-500" />}
                <span className="truncate">{file.name}</span>
            </div>
        ))}
    </div>
  );
};
