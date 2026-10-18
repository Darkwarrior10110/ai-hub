import { createContext, useContext, useState, useEffect, useCallback, useRef } from 'react';
import type { ReactNode } from 'react';
import type { Theme } from '../types';
import {
  gatewayService,
  type ProviderId,
  type ManagedApplication,
  type RuntimeSettings,
  type DiscoverySnapshot,
  type LaunchResult,
  type ResourceSnapshot,
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
  systemResources: ResourceSnapshot | null;
  isScanning: boolean;
  isLaunching: boolean;
  launchingProviderId: ProviderId | null;
  isStopping: Record<ProviderId, boolean>;
  isSwitching: boolean;
  switchingTargetId: ProviderId | null;
  switchError: string | null;
  clearSwitchError: () => void;
  lastLaunchResult: LaunchResult | null;
  scanApplications: () => Promise<void>;
  launchProvider: (id: ProviderId) => Promise<LaunchResult>;
  stopProvider: (id: ProviderId) => Promise<void>;
  switchProvider: (id: ProviderId) => Promise<ManagedApplication>;
  refreshProviderStates: () => Promise<void>;
  refreshSystemResources: () => Promise<void>;
  reconcileStates: () => Promise<void>;
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
  const [systemResources, setSystemResources] = useState<ResourceSnapshot | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [isLaunching, setIsLaunching] = useState(false);
  const [launchingProviderId, setLaunchingProviderId] = useState<ProviderId | null>(null);
  const [isStopping, setIsStopping] = useState<Record<ProviderId, boolean>>({
    chatgpt: false,
    gemini: false,
    claude: false,
    grok: false,
    perplexity: false,
    kimi: false,
  });
  const [isSwitching, setIsSwitching] = useState(false);
  const [switchingTargetId, setSwitchingTargetId] = useState<ProviderId | null>(null);
  const [switchError, setSwitchError] = useState<string | null>(null);
  const [lastLaunchResult, setLastLaunchResult] = useState<LaunchResult | null>(null);

  // Request sequencing and mutation timestamp refs to drop stale asynchronous responses
  const stateVersionRef = useRef<number>(0);
  const lastStateMutationTimestampRef = useRef<number>(Date.now());
  const isSwitchingRef = useRef<boolean>(false);
  const isLaunchingRef = useRef<boolean>(false);
  const isAnyStoppingRef = useRef<boolean>(false);

  useEffect(() => {
    isSwitchingRef.current = isSwitching;
  }, [isSwitching]);

  useEffect(() => {
    isLaunchingRef.current = isLaunching;
  }, [isLaunching]);

  useEffect(() => {
    isAnyStoppingRef.current = Object.values(isStopping).some(Boolean);
  }, [isStopping]);

  const clearSwitchError = useCallback(() => {
    setSwitchError(null);
  }, []);

  const refreshSystemResources = useCallback(async () => {
    try {
      const res = await gatewayService.getSystemResources();
      setSystemResources(res);
    } catch {
      // Non-blocking fallback
    }
  }, []);

  const refreshProviderStates = useCallback(async () => {
    const reqVersion = ++stateVersionRef.current;
    const reqTime = Date.now();
    try {
      const states = await gatewayService.reconcileProviderStates();
      if (
        reqVersion < stateVersionRef.current ||
        reqTime < lastStateMutationTimestampRef.current
      ) {
        return;
      }

      setManagedApps((prev) => {
        let changed = false;
        const next = { ...prev };
        for (const [key, val] of Object.entries(states)) {
          const pid = key as ProviderId;
          const prevApp = prev[pid];
          if (
            !prevApp ||
            prevApp.lifecycle_state !== val.lifecycle_state ||
            prevApp.is_active !== val.is_active ||
            prevApp.process?.pid !== val.process?.pid ||
            prevApp.error_message !== val.error_message
          ) {
            next[pid] = val;
            changed = true;
          }
        }
        return changed ? next : prev;
      });

      const active = await gatewayService.getActiveProvider();
      if (active && (active as ProviderId) in states) {
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
      await refreshSystemResources();
    } catch {
      // Non-blocking discovery fallback
    } finally {
      setIsScanning(false);
    }
  }, [refreshProviderStates, refreshSystemResources]);

  const reconcileStates = useCallback(async () => {
    const reqVersion = ++stateVersionRef.current;
    const reqTime = Date.now();
    try {
      const states = await gatewayService.reconcileProviderStates();
      // Drop stale response if a newer mutation occurred while in flight
      if (
        reqVersion < stateVersionRef.current ||
        reqTime < lastStateMutationTimestampRef.current
      ) {
        return;
      }
      setManagedApps((prev) => {
        let changed = false;
        const next = { ...prev };
        for (const [key, val] of Object.entries(states)) {
          const pid = key as ProviderId;
          const prevApp = prev[pid];
          if (
            !prevApp ||
            prevApp.lifecycle_state !== val.lifecycle_state ||
            prevApp.is_active !== val.is_active ||
            prevApp.process?.pid !== val.process?.pid ||
            prevApp.error_message !== val.error_message
          ) {
            next[pid] = val;
            changed = true;
          }
        }
        return changed ? next : prev;
      });
    } catch {
      // Non-blocking fallback
    }
  }, []);

  useEffect(() => {
    refreshProviderStates().then(() => {
      scanApplications();
      refreshSystemResources();
      reconcileStates();
    });
  }, [refreshProviderStates, scanApplications, refreshSystemResources, reconcileStates]);

  // Bounded non-overlapping lifecycle reconciliation and resource monitoring poll (3 seconds interval)
  useEffect(() => {
    let isMounted = true;
    let timerId: ReturnType<typeof setTimeout> | null = null;
    let inFlight = false;

    const poll = async () => {
      if (!isMounted || inFlight) return;
      if (isSwitchingRef.current || isLaunchingRef.current || isAnyStoppingRef.current) {
        timerId = setTimeout(poll, 1500);
        return;
      }
      inFlight = true;
      try {
        await refreshSystemResources();
        await reconcileStates();
      } finally {
        inFlight = false;
        if (isMounted) {
          timerId = setTimeout(poll, 3000);
        }
      }
    };

    poll();

    return () => {
      isMounted = false;
      if (timerId) clearTimeout(timerId);
    };
  }, [refreshSystemResources, reconcileStates]);

  const setSelectedProvider = useCallback((p: ProviderId) => {
    setSelectedProviderState(p);
    gatewayService.setActiveProvider(p).catch(() => {});
  }, []);

  const launchProvider = useCallback(async (id: ProviderId): Promise<LaunchResult> => {
    setIsLaunching(true);
    setLaunchingProviderId(id);
    lastStateMutationTimestampRef.current = Date.now();
    stateVersionRef.current += 1;
    try {
      const result = await gatewayService.launchProvider(id);
      setLastLaunchResult(result);
      lastStateMutationTimestampRef.current = Date.now();
      stateVersionRef.current += 1;
      await refreshProviderStates();
      await refreshSystemResources();
      return result;
    } catch (err) {
      lastStateMutationTimestampRef.current = Date.now();
      stateVersionRef.current += 1;
      await refreshProviderStates();
      await refreshSystemResources();
      throw err;
    } finally {
      setIsLaunching(false);
      setLaunchingProviderId(null);
    }
  }, [refreshProviderStates, refreshSystemResources]);

  const stopProvider = useCallback(async (id: ProviderId) => {
    setIsStopping((prev) => ({ ...prev, [id]: true }));
    lastStateMutationTimestampRef.current = Date.now();
    stateVersionRef.current += 1;
    try {
      const updated = await gatewayService.stopProvider(id);
      lastStateMutationTimestampRef.current = Date.now();
      stateVersionRef.current += 1;
      setManagedApps((prev) => ({ ...prev, [id]: updated }));
      await refreshProviderStates();
      await refreshSystemResources();
    } catch (err) {
      lastStateMutationTimestampRef.current = Date.now();
      stateVersionRef.current += 1;
      await refreshProviderStates();
      await refreshSystemResources();
      throw err;
    } finally {
      setIsStopping((prev) => ({ ...prev, [id]: false }));
    }
  }, [refreshProviderStates, refreshSystemResources]);

  const switchProvider = useCallback(async (id: ProviderId): Promise<ManagedApplication> => {
    setIsSwitching(true);
    setSwitchingTargetId(id);
    setSwitchError(null);
    lastStateMutationTimestampRef.current = Date.now();
    stateVersionRef.current += 1;
    try {
      const updated = await gatewayService.switchProvider(id);
      setSelectedProviderState(id);
      lastStateMutationTimestampRef.current = Date.now();
      stateVersionRef.current += 1;

      // Reconcile all affected providers: target is now Running, previous running apps are Stopped
      await refreshProviderStates();
      await refreshSystemResources();
      return updated;
    } catch (err: unknown) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setSwitchError(errMsg);
      lastStateMutationTimestampRef.current = Date.now();
      stateVersionRef.current += 1;
      await refreshProviderStates();
      await refreshSystemResources();
      throw err;
    } finally {
      setIsSwitching(false);
      setSwitchingTargetId(null);
    }
  }, [refreshProviderStates, refreshSystemResources]);

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
        systemResources,
        isScanning,
        isLaunching,
        launchingProviderId,
        isStopping,
        isSwitching,
        switchingTargetId,
        switchError,
        clearSwitchError,
        lastLaunchResult,
        scanApplications,
        launchProvider,
        stopProvider,
        switchProvider,
        refreshProviderStates,
        refreshSystemResources,
        reconcileStates,
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
