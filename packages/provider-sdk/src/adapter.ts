import { ConnectionState } from './types';
import { ProviderCapabilities } from './capabilities';
import { AuthenticationStrategy } from './auth';

export interface ProviderInfo {
  id: string;
  name: string;
  version: string;
}

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface ProviderAdapter {
  readonly info: ProviderInfo;
  readonly capabilities: ProviderCapabilities;
  readonly supportedAuthStrategies: AuthenticationStrategy[];
  
  getConnectionState(): ConnectionState;
  
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  restoreConnection(): Promise<void>;
  
  sendMessage(message: Message): Promise<Message>;
}
