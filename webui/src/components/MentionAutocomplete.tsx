import React, { useEffect, useState, useMemo } from 'react';
import {
  Command,
  CommandGroup,
  CommandItem,
  CommandList,
} from '@/components/ui/command';
import { api } from '../lib/api';
import { File, Folder } from 'lucide-react';

interface MentionAutocompleteProps {
  input: string;
  onSelect: (mention: string) => void;
  isOpen: boolean;
  setIsOpen: (open: boolean) => void;
}

export const MentionAutocomplete: React.FC<MentionAutocompleteProps> = ({
  input,
  onSelect,
  isOpen,
  setIsOpen,
}) => {
  const [entries, setEntries] = useState<Array<{ name: string, type: string }>>([]);
  const [wikiPages, setWikiPages] = useState<string[]>([]);

  useEffect(() => {
    const fetchData = async () => {
       try {
           const [fileData, wikiData] = await Promise.all([
               api.listWorkspaceFiles(),
               api.listWikiPages()
           ]);
           setEntries(fileData.entries);
           setWikiPages(wikiData.pages);
       } catch (e) {
           console.error("Failed to fetch autocomplete data", e);
       }
    };
    fetchData();
  }, []);

  const searchTerm = useMemo(() => {
    const lastWord = input.split(/\s/).pop() || "";
    if (lastWord.startsWith('@')) {
        return lastWord.slice(1).toLowerCase();
    }
    return null;
  }, [input]);

  const filteredEntries = useMemo(() => {
    if (searchTerm === null) return [];
    return entries.filter(e => e.name.toLowerCase().includes(searchTerm)).slice(0, 50);
  }, [entries, searchTerm]);

  const filteredWiki = useMemo(() => {
    if (searchTerm === null) return [];
    return wikiPages.filter(p => p.toLowerCase().includes(searchTerm)).slice(0, 50);
  }, [wikiPages, searchTerm]);

  useEffect(() => {
    const shouldBeOpen = searchTerm !== null && (filteredEntries.length > 0 || filteredWiki.length > 0);
    if (isOpen !== shouldBeOpen) {
        setIsOpen(shouldBeOpen);
    }
  }, [filteredEntries, filteredWiki, searchTerm, isOpen, setIsOpen]);

  if (!isOpen) return null;

  return (
    <div className="absolute bottom-full left-0 w-full mb-2 z-50">
      <div className="max-w-3xl mx-auto bg-white rounded-lg shadow-lg border border-input overflow-hidden">
        <Command>
          <CommandList className="max-h-60 overflow-y-auto">
            {filteredEntries.length > 0 && (
              <CommandGroup heading="Workspace Files & Folders">
                {filteredEntries.map((e) => (
                  <CommandItem
                    key={e.name}
                    onSelect={() => onSelect(`@${e.name}`)}
                    className="cursor-pointer"
                  >
                    {e.type === 'folder' ? <Folder size={14} className="text-blue-500" /> : <File size={14} className="text-gray-500" />}
                    <span className="font-mono text-xs">{e.name}</span>
                  </CommandItem>
                ))}
              </CommandGroup>
            )}
            {filteredWiki.length > 0 && (
              <CommandGroup heading="Wiki Pages">
                {filteredWiki.map((p) => (
                  <CommandItem
                    key={p}
                    onSelect={() => onSelect(`@wiki/${p}`)}
                    className="cursor-pointer"
                  >
                    <span className="font-medium">{p}</span>
                  </CommandItem>
                ))}
              </CommandGroup>
            )}
          </CommandList>
        </Command>
      </div>
    </div>
  );
};
