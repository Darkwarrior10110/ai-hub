import { ProviderAdapter, ProviderNotFound } from '@ai-hub/provider-sdk';

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
