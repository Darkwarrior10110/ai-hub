import { createContext, useContext, useState, useEffect, useCallback, useSyncExternalStore } from 'react';
import type { ReactNode } from 'react';
import type { Theme, ProviderId } from '../types';
import { providerManager } from './registry';

interface UIState {
  theme: Theme;
  setTheme: (t: Theme) => void;
  selectedProvider: ProviderId;
  setSelectedProvider: (p: ProviderId) => void;
  isSettingsOpen: boolean;
  setSettingsOpen: (v: boolean) => void;
  isSidebarCollapsed: boolean;
  toggleSidebar: () => void;
  connectProvider: (id: string) => Promise<void>;
  disconnectProvider: (id: string) => Promise<void>;
}

const UIContext = createContext<UIState | undefined>(undefined);

export function UIProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<Theme>('dark');
  const [isSettingsOpen, setSettingsOpen] = useState(false);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);

  // Sync state from ProviderManager
  useSyncExternalStore(
    (onStoreChange) => providerManager.subscribe(onStoreChange),
    () => providerManager.getActiveProvider()
  );
  
  const selectedProvider = (providerManager.getActiveProvider()?.info.id as ProviderId) || 'chatgpt';

  const setSelectedProvider = useCallback((p: ProviderId) => {
    providerManager.selectProvider(p);
  }, []);

  const connectProvider = useCallback(async (id: string) => {
    await providerManager.connectProvider(id);
  }, []);

  const disconnectProvider = useCallback(async (id: string) => {
    await providerManager.disconnectProvider(id);
  }, []);

  const toggleSidebar = useCallback(() => {
    setIsSidebarCollapsed(prev => !prev);
  }, []);

  useEffect(() => {
    const applyTheme = (t: Theme) => {
      if (t === 'system') {
        const dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
      } else {
        document.documentElement.setAttribute('data-theme', t);
      }
    };
    applyTheme(theme);

    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const listener = () => { if (theme === 'system') applyTheme('system'); };
    mq.addEventListener('change', listener);
    return () => mq.removeEventListener('change', listener);
  }, [theme]);

  // Close settings with Escape
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isSettingsOpen) setSettingsOpen(false);
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [isSettingsOpen]);

  return (
    <UIContext.Provider value={{
      theme, setTheme,
      selectedProvider, setSelectedProvider,
      isSettingsOpen, setSettingsOpen,
      isSidebarCollapsed, toggleSidebar,
      connectProvider, disconnectProvider
    }}>
      {children}
    </UIContext.Provider>
  );
}

export function useUI() {
  const ctx = useContext(UIContext);
  if (!ctx) throw new Error('useUI must be used within UIProvider');
  return ctx;
}
