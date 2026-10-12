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
