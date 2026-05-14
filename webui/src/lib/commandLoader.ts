export interface CommandDefinition {
  name: string;
  template: string; // The backend-ready prompt string with {args} placeholder
}

let registry: CommandDefinition[] = [];

export async function loadCommands(): Promise<void> {
  try {
    const response = await fetch('/api/commands');
    if (!response.ok) throw new Error('Failed to fetch commands');
    
    const data: Record<string, string> = await response.json();
    registry = Object.entries(data).map(([name, template]) => ({
      name,
      template,
    }));
  } catch (error) {
    console.error('Failed to load slash commands:', error);
  }
}

export function getRegistry(): CommandDefinition[] {
  return registry;
}

export function processSlashCommand(input: string): string {
  if (!input.startsWith('/')) return input;

  const parts = input.trim().split(' ');
  const commandName = parts[0];
  const args = parts.slice(1).join(' ');

  const command = registry.find(cmd => cmd.name === commandName);

  if (command) {
    return command.template.replace('{args}', args);
  }

  return input;
}
