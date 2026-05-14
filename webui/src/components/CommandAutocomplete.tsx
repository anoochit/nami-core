import React, { useEffect, useState, useMemo } from 'react';
import {
  Command,
  CommandGroup,
  CommandItem,
  CommandList,
} from '@/components/ui/command';
import { getRegistry, type CommandDefinition,  } from '../lib/commandLoader';

interface CommandAutocompleteProps {
  input: string;
  onSelect: (command: string) => void;
  isOpen: boolean;
  setIsOpen: (open: boolean) => void;
}

export const CommandAutocomplete: React.FC<CommandAutocompleteProps> = ({
  input,
  onSelect,
  isOpen,
  setIsOpen,
}) => {
  // Initialize with registry data since it's pre-loaded in main.tsx
  const [commands] = useState<CommandDefinition[]>(() => getRegistry());

  const filteredCommands = useMemo(() => {
    if (input.startsWith('/') && !input.includes(' ')) {
      const searchTerm = input.toLowerCase();
      return commands.filter((cmd) =>
        cmd.name.toLowerCase().startsWith(searchTerm)
      );
    }
    return [];
  }, [input, commands]);

  useEffect(() => {
    const shouldBeOpen = filteredCommands.length > 0;
    if (isOpen !== shouldBeOpen) {
        setIsOpen(shouldBeOpen);
    }
  }, [filteredCommands, isOpen, setIsOpen]);

  if (!isOpen) return null;

  return (
    <div className="absolute bottom-full left-0 w-full mb-2 z-50">
      <div className="max-w-3xl mx-auto bg-popover rounded-lg shadow-lg border border-input overflow-hidden">
        <Command>
          <CommandList className="max-h-60 overflow-y-auto">
            <CommandGroup heading="Slash Commands">
              {filteredCommands.map((cmd) => (
                <CommandItem
                  key={cmd.name}
                  onSelect={() => onSelect(cmd.name)}
                  className="cursor-pointer"
                >
                  <div className="flex flex-col">
                    <span className="font-bold text-primary">{cmd.name}</span>
                    {cmd.help && (
                      <span className="text-xs text-muted-foreground">
                        {cmd.help}
                      </span>
                    )}
                  </div>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </div>
    </div>
  );
};
