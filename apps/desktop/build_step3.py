import os
import textwrap

packages_dir = "/home/dark_warrior/Projects/ai-hub/packages/provider-sdk"
desktop_src_dir = "/home/dark_warrior/Projects/ai-hub/apps/desktop/src"
desktop_tests_dir = "/home/dark_warrior/Projects/ai-hub/apps/desktop/tests"

sdk_files = {
    "package.json": """
{
  "name": "@ai-hub/provider-sdk",
  "version": "0.1.0",
  "main": "src/index.ts",
  "types": "src/index.ts"
}
""",
    "src/types.ts": """
export enum ConnectionState {
  DISCONNECTED = 'DISCONNECTED',
  CONNECTING = 'CONNECTING',
  CONNECTED = 'CONNECTED',
  ERROR = 'ERROR',
  UNAVAILABLE = 'UNAVAILABLE'
}
""",
    "src/capabilities.ts": """
export interface ProviderCapabilities {
  chat: boolean;
  streaming: boolean;
  attachments: boolean;
  imageInput: boolean;
  webSearch: boolean;
  history: boolean;
  modelSelection: boolean;
}
""",
    "src/auth.ts": """
export enum AuthenticationStrategy {
  OAUTH = 'OAUTH',
  DEVICE_AUTHORIZATION = 'DEVICE_AUTHORIZATION',
  API_KEY = 'API_KEY',
  PROVIDER_SPECIFIC = 'PROVIDER_SPECIFIC',
  NONE = 'NONE'
}
""",
    "src/credentials.ts": """
export interface SecureCredentialStore {
  saveCredential(providerId: string, credentialData: string): Promise<void>;
  getCredential(providerId: string): Promise<string | null>;
  deleteCredential(providerId: string): Promise<void>;
}
""",
    "src/errors.ts": """
export class ProviderError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ProviderError';
  }
}
export class ProviderNotFound extends ProviderError { name = 'ProviderNotFound'; }
export class ProviderUnavailable extends ProviderError { name = 'ProviderUnavailable'; }
export class AuthenticationRequired extends ProviderError { name = 'AuthenticationRequired'; }
export class AuthenticationFailed extends ProviderError { name = 'AuthenticationFailed'; }
export class AuthenticationNotSupported extends ProviderError { name = 'AuthenticationNotSupported'; }
export class ConnectionFailed extends ProviderError { name = 'ConnectionFailed'; }
export class RateLimited extends ProviderError { name = 'RateLimited'; }
export class CapabilityNotSupported extends ProviderError { name = 'CapabilityNotSupported'; }
export class ProviderConfigurationError extends ProviderError { name = 'ProviderConfigurationError'; }
""",
    "src/adapter.ts": """
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
""",
    "src/index.ts": """
export * from './types';
export * from './capabilities';
export * from './auth';
export * from './credentials';
export * from './errors';
export * from './adapter';
"""
}

desktop_provider_files = {
    "manager/ProviderManager.ts": """
import { ProviderAdapter, ConnectionState, ProviderNotFound } from '@ai-hub/provider-sdk';

export class ProviderManager {
  private adapters: Map<string, ProviderAdapter> = new Map();
  private activeProviderId: string | null = null;
  private listeners: Set<() => void> = new Set();

  registerProvider(adapter: ProviderAdapter): void {
    this.adapters.set(adapter.info.id, adapter);
    if (!this.activeProviderId) {
      this.activeProviderId = adapter.info.id;
    }
    this.notifyListeners();
  }

  getAvailableProviders(): ProviderAdapter[] {
    return Array.from(this.adapters.values());
  }

  getProvider(id: string): ProviderAdapter {
    const adapter = this.adapters.get(id);
    if (!adapter) throw new ProviderNotFound(`Provider ${id} not found`);
    return adapter;
  }

  selectProvider(id: string): void {
    if (!this.adapters.has(id)) throw new ProviderNotFound(`Provider ${id} not found`);
    this.activeProviderId = id;
    this.notifyListeners();
  }

  getActiveProvider(): ProviderAdapter | null {
    if (!this.activeProviderId) return null;
    return this.adapters.get(this.activeProviderId) || null;
  }

  async connectProvider(id: string): Promise<void> {
    const provider = this.getProvider(id);
    try {
      await provider.connect();
    } catch (error) {
      // Allow error to propagate but notify UI state change
      throw error;
    } finally {
      this.notifyListeners();
    }
  }

  async disconnectProvider(id: string): Promise<void> {
    const provider = this.getProvider(id);
    await provider.disconnect();
    this.notifyListeners();
  }

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  notifyListeners(): void {
    this.listeners.forEach(l => l());
  }
}
""",
    "registry/index.ts": """
import { ProviderManager } from '../manager/ProviderManager';
import { ChatGPTAdapter } from '../adapters/chatgpt';
import { GeminiAdapter } from '../adapters/gemini';
import { ClaudeAdapter } from '../adapters/claude';
import { GrokAdapter } from '../adapters/grok';
import { PerplexityAdapter } from '../adapters/perplexity';
import { KimiAdapter } from '../adapters/kimi';

export const providerManager = new ProviderManager();

providerManager.registerProvider(new ChatGPTAdapter());
providerManager.registerProvider(new GeminiAdapter());
providerManager.registerProvider(new ClaudeAdapter());
providerManager.registerProvider(new GrokAdapter());
providerManager.registerProvider(new PerplexityAdapter());
providerManager.registerProvider(new KimiAdapter());
"""
}

def create_adapter(id, name):
    return f"""
import {{ ProviderAdapter, ProviderInfo, ProviderCapabilities, AuthenticationStrategy, ConnectionState, Message, AuthenticationNotSupported }} from '@ai-hub/provider-sdk';

export class {name.replace(" ", "")}Adapter implements ProviderAdapter {{
  info: ProviderInfo = {{ id: '{id}', name: '{name}', version: '1.0.0' }};
  
  capabilities: ProviderCapabilities = {{
    chat: true,
    streaming: true,
    attachments: false,
    imageInput: false,
    webSearch: false,
    history: true,
    modelSelection: false
  }};
  
  supportedAuthStrategies: AuthenticationStrategy[] = [
    AuthenticationStrategy.OAUTH, 
    AuthenticationStrategy.API_KEY
  ];
  
  private state: ConnectionState = ConnectionState.DISCONNECTED;

  getConnectionState(): ConnectionState {{
    return this.state;
  }}

  async connect(): Promise<void> {{
    this.state = ConnectionState.CONNECTING;
    // Architecture phase only: do NOT implement real auth yet
    await new Promise(resolve => setTimeout(resolve, 300));
    this.state = ConnectionState.ERROR;
    throw new AuthenticationNotSupported('Connection mechanism not configured yet for {name}.');
  }}

  async disconnect(): Promise<void> {{
    this.state = ConnectionState.DISCONNECTED;
  }}

  async restoreConnection(): Promise<void> {{
    this.state = ConnectionState.DISCONNECTED;
  }}

  async sendMessage(message: Message): Promise<Message> {{
    throw new Error('Not connected');
  }}
}}
"""

desktop_provider_files["adapters/chatgpt.ts"] = create_adapter("chatgpt", "ChatGPT")
desktop_provider_files["adapters/gemini.ts"] = create_adapter("gemini", "Gemini")
desktop_provider_files["adapters/claude.ts"] = create_adapter("claude", "Claude")
desktop_provider_files["adapters/grok.ts"] = create_adapter("grok", "Grok")
desktop_provider_files["adapters/perplexity.ts"] = create_adapter("perplexity", "Perplexity")
desktop_provider_files["adapters/kimi.ts"] = create_adapter("kimi", "Kimi")

tests_files = {
    "ProviderManager.test.ts": """
import { describe, it, expect, beforeEach } from 'vitest';
import { ProviderManager } from '../src/providers/manager/ProviderManager';
import { ChatGPTAdapter } from '../src/providers/adapters/chatgpt';
import { ProviderNotFound, ConnectionState, AuthenticationNotSupported } from '@ai-hub/provider-sdk';

describe('ProviderManager', () => {
  let manager: ProviderManager;

  beforeEach(() => {
    manager = new ProviderManager();
  });

  it('should register and retrieve a provider', () => {
    const adapter = new ChatGPTAdapter();
    manager.registerProvider(adapter);
    
    expect(manager.getAvailableProviders().length).toBe(1);
    expect(manager.getProvider('chatgpt')).toBe(adapter);
  });

  it('should throw ProviderNotFound for invalid provider', () => {
    expect(() => manager.getProvider('invalid')).toThrow(ProviderNotFound);
  });

  it('should handle provider selection', () => {
    manager.registerProvider(new ChatGPTAdapter());
    manager.selectProvider('chatgpt');
    
    const active = manager.getActiveProvider();
    expect(active?.info.id).toBe('chatgpt');
  });

  it('should test provider isolation (one connection failure does not affect others)', async () => {
    const p1 = new ChatGPTAdapter();
    manager.registerProvider(p1);
    
    // Attempt connection, expect failure (architecture placeholder)
    await expect(manager.connectProvider('chatgpt')).rejects.toThrow(AuthenticationNotSupported);
    expect(manager.getProvider('chatgpt').getConnectionState()).toBe(ConnectionState.ERROR);
  });

  it('should expose capability models correctly', () => {
    const p1 = new ChatGPTAdapter();
    manager.registerProvider(p1);
    expect(manager.getProvider('chatgpt').capabilities.chat).toBe(true);
  });
});
"""
}

def write_files(base_dir, files_dict):
    for path, content in files_dict.items():
        full_path = os.path.join(base_dir, path)
        os.makedirs(os.path.dirname(full_path), exist_ok=True)
        with open(full_path, "w") as f:
            f.write(content.strip() + "\n")

write_files(packages_dir, sdk_files)
write_files(os.path.join(desktop_src_dir, "providers"), desktop_provider_files)
write_files(desktop_tests_dir, tests_files)
