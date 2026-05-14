export interface Message {
  id: string;
  sender: 'user' | 'agent';
  text: string;
  toolCall?: { 
    name: string; 
    args?: Record<string, unknown>; 
    status: 'pending' | 'complete'; 
    result?: unknown 
  };
}

export interface Thread {
  id: string;
  title: string;
  messages: Message[];
  sessionId?: string;
}

export interface Session {
  session_id: string;
}

export interface AgentContent {
    role: 'user' | 'agent';
    parts: Array<{
        text?: string;
        name?: string;
        args?: Record<string, unknown>;
        functionResponse?: {
            response: unknown;
        };
    }>;
}

export interface AgentEvent {
    content?: {
        parts?: Array<{
            text?: string;
            name?: string;
            args?: Record<string, unknown>;
            functionResponse?: {
                response: unknown;
            };
        }>;
    };
}
