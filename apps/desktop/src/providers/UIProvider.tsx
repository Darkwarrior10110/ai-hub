import { createContext, useContext, useState, useEffect, useCallback } from 'react';
import type { ReactNode } from 'react';
import type { Theme } from '../types';
import {
  gatewayService,
  type ProviderId,
  type ManagedApplication,
  type RuntimeSettings,
  type DiscoverySnapshot,
} from '../services/gateway';

interface UIState {
  theme: Theme;
  setTheme: (t: Theme) => void;
  selectedProvider: ProviderId;
  setSelectedProvider: (p: ProviderId) => void;
  isSettingsOpen: boolean;
  setSettingsOpen: (v: boolean) => void;
  isSidebarCollapsed: boolean;
  toggleSidebar: () => void;
  managedApps: Record<ProviderId, ManagedApplication>;
  runtimeSettings: RuntimeSettings;
  discoverySnapshot: DiscoverySnapshot | null;
  isScanning: boolean;
  scanApplications: () => Promise<void>;
  launchProvider: (id: ProviderId) => Promise<void>;
  stopProvider: (id: ProviderId) => Promise<void>;
  switchProvider: (id: ProviderId) => Promise<void>;
  refreshProviderStates: () => Promise<void>;
  updateSettings: (settings: RuntimeSettings) => Promise<void>;
}

const UIContext = createContext<UIState | undefined>(undefined);

const initialManagedApps: Record<ProviderId, ManagedApplication> = {
  chatgpt: { provider_id: 'chatgpt', lifecycle_state: 'unknown', is_active: false, last_state_change_ms: 0 },
  gemini: { provider_id: 'gemini', lifecycle_state: 'unknown', is_active: false, last_state_change_ms: 0 },
  claude: { provider_id: 'claude', lifecycle_state: 'unknown', is_active: false, last_state_change_ms: 0 },
  grok: { provider_id: 'grok', lifecycle_state: 'unknown', is_active: false, last_state_change_ms: 0 },
  perplexity: { provider_id: 'perplexity', lifecycle_state: 'unknown', is_active: false, last_state_change_ms: 0 },
  kimi: { provider_id: 'kimi', lifecycle_state: 'unknown', is_active: false, last_state_change_ms: 0 },
};

const initialRuntimeSettings: RuntimeSettings = {
  resource_saver_mode: true,
  confirm_before_switching: true,
  graceful_shutdown_timeout_secs: 5,
  allow_force_termination: false,
  auto_start_on_boot: false,
  metrics_polling_interval_secs: 3,
};

export function UIProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<Theme>('dark');
  const [isSettingsOpen, setSettingsOpen] = useState(false);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const [selectedProvider, setSelectedProviderState] = useState<ProviderId>('chatgpt');
  const [managedApps, setManagedApps] = useState<Record<ProviderId, ManagedApplication>>(initialManagedApps);
  const [runtimeSettings, setRuntimeSettings] = useState<RuntimeSettings>(initialRuntimeSettings);
  const [discoverySnapshot, setDiscoverySnapshot] = useState<DiscoverySnapshot | null>(null);
  const [isScanning, setIsScanning] = useState(false);

  const refreshProviderStates = useCallback(async () => {
    try {
      const providers = await gatewayService.listProviders();
      const updated: Record<ProviderId, ManagedApplication> = { ...initialManagedApps };

      await Promise.all(
        providers.map(async (p) => {
          try {
            const state = await gatewayService.getProviderState(p.id);
            updated[p.id] = state;
          } catch {
            // Keep existing state on error
          }
        })
      );

      setManagedApps(updated);

      const active = await gatewayService.getActiveProvider();
      if (active && (active as ProviderId) in updated) {
        setSelectedProviderState(active as ProviderId);
      }

      const settings = await gatewayService.getRuntimeSettings();
      setRuntimeSettings(settings);
    } catch {
      // Non-blocking initialization fallback
    }
  }, []);

  const scanApplications = useCallback(async () => {
    setIsScanning(true);
    try {
      const snapshot = await gatewayService.discoverAllProviders();
      setDiscoverySnapshot(snapshot);
      await refreshProviderStates();
    } catch {
      // Non-blocking discovery fallback
    } finally {
      setIsScanning(false);
    }
  }, [refreshProviderStates]);

  useEffect(() => {
    refreshProviderStates().then(() => {
      scanApplications();
    });
  }, [refreshProviderStates, scanApplications]);

  const setSelectedProvider = useCallback((p: ProviderId) => {
    setSelectedProviderState(p);
    gatewayService.setActiveProvider(p).catch(() => {});
  }, []);

  const launchProvider = useCallback(async (id: ProviderId) => {
    const updated = await gatewayService.launchProvider(id);
    setManagedApps((prev) => ({ ...prev, [id]: updated }));
  }, []);

  const stopProvider = useCallback(async (id: ProviderId) => {
    const updated = await gatewayService.stopProvider(id);
    setManagedApps((prev) => ({ ...prev, [id]: updated }));
  }, []);

  const switchProvider = useCallback(async (id: ProviderId) => {
    const updated = await gatewayService.switchProvider(id);
    setManagedApps((prev) => ({ ...prev, [id]: updated }));
  }, []);

  const updateSettings = useCallback(async (settings: RuntimeSettings) => {
    const updated = await gatewayService.updateRuntimeSettings(settings);
    setRuntimeSettings(updated);
  }, []);

  const toggleSidebar = useCallback(() => {
    setIsSidebarCollapsed((prev) => !prev);
  }, []);

  // Theme application
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
    const listener = () => {
      if (theme === 'system') applyTheme('system');
    };
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
    <UIContext.Provider
      value={{
        theme,
        setTheme,
        selectedProvider,
        setSelectedProvider,
        isSettingsOpen,
        setSettingsOpen,
        isSidebarCollapsed,
        toggleSidebar,
        managedApps,
        runtimeSettings,
        discoverySnapshot,
        isScanning,
        scanApplications,
        launchProvider,
        stopProvider,
        switchProvider,
        refreshProviderStates,
        updateSettings,
      }}
    >
      {children}
    </UIContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useUI() {
  const ctx = useContext(UIContext);
  if (!ctx) throw new Error('useUI must be used within UIProvider');
  return ctx;
}
