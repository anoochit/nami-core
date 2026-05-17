import { getHeaders } from './api';

export interface CommandDefinition {
  name: string;
  template: string; // The backend-ready prompt string with {args} placeholder
  help?: string;    // Help description for the command
}

let registry: CommandDefinition[] = [];

export async function loadCommands(): Promise<void> {
  try {
    const response = await fetch('/api/commands', {
        headers: getHeaders()
    });
    if (!response.ok) throw new Error('Failed to fetch commands');
    
    const data: Record<string, { template: string, help: string }> = await response.json();
    registry = Object.entries(data).map(([name, cmd]) => ({
      name,
      template: cmd.template,
      help: cmd.help,
    }));

    // Add built-in help command if not present
    if (!registry.find(cmd => cmd.name === '/?')) {
      registry.push({
          name: '/?',
          template: 'HELP_COMMAND',
          help: 'Show this help message'
      });
    }

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

  if (commandName === '/?') {
      return generateHelpMessage();
  }

  const command = registry.find(cmd => cmd.name === commandName);

  if (command) {
    let result = command.template;
    
    // Support advanced partitioning like in the backend (goal | stop/cron)
    const argParts = args.split('|').map(s => s.trim());
    
    if (argParts[0]) result = result.replace('{goal}', argParts[0]);
    if (argParts[1]) {
        result = result.replace('{cron}', argParts[1]);
        result = result.replace('{stop}', argParts[1]);
    }
    
    result = result.replace('{args}', args);
    // Unique task ID placeholder
    result = result.replace('{uuid}', Math.random().toString(36).substring(2, 10));

    return result;
  }

  return input;
}

function generateHelpMessage(): string {
    let help = "### Available Slash Commands\n\n";
    registry.forEach(cmd => {
        help += `* **${cmd.name}**: ${cmd.help || 'No description'}\n`;
    });
    return help;
}
