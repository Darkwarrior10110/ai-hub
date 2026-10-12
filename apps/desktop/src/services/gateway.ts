import { invoke } from '@tauri-apps/api/core';

export type ProviderId = 'chatgpt' | 'gemini' | 'claude' | 'grok' | 'perplexity' | 'kimi';

export type ApplicationLifecycleState =
  | 'not_installed'
  | 'installed'
  | 'ready'
  | 'starting'
  | 'running'
  | 'stopping'
  | 'switching'
  | 'error'
  | 'unknown';

export type ProcessOwnership =
  | 'started_by_ai_hub'
  | 'already_running_externally'
  | 'discovered_running'
  | 'ownership_unknown';

export interface ProcessIdentity {
  pid: number;
  process_name: string;
  start_time_ms?: number;
  ownership: ProcessOwnership;
}

export interface ManagedApplication {
  provider_id: ProviderId;
  lifecycle_state: ApplicationLifecycleState;
  process?: ProcessIdentity | null;
  is_active: boolean;
  last_state_change_ms: number;
  error_message?: string | null;
}

export interface ProviderDefinition {
  id: ProviderId;
  name: string;
  description: string;
  vendor: string;
  accent_color: string;
  supported_platforms: ('linux' | 'windows' | 'macos')[];
  candidate_snap_names: string[];
  candidate_desktop_ids: string[];
  candidate_executables: string[];
}

export interface RuntimeSettings {
  resource_saver_mode: boolean;
  confirm_before_switching: boolean;
  graceful_shutdown_timeout_secs: number;
  allow_force_termination: boolean;
  auto_start_on_boot: boolean;
  metrics_polling_interval_secs: number;
}

export interface InstalledApplication {
  provider_id: ProviderId;
  display_name: string;
  version?: string | null;
  launch_type: 'snap' | 'desktop_entry' | 'executable' | 'app_image' | 'windows_app';
  executable_path: string;
  launch_args: string[];
  icon_path?: string | null;
  package_id?: string | null;
}

export type MatchConfidence = 'confirmed' | 'ambiguous' | 'none';

export type DiscoveryStatus =
  | 'confirmed'
  | 'ambiguous'
  | 'not_installed'
  | 'unavailable'
  | 'failed';

export type ApplicationSource = 'snap' | 'desktop_entry' | 'binary' | 'windows_registry';

export interface ProviderDiscoveryResult {
  provider_id: ProviderId;
  status: DiscoveryStatus;
  match_confidence: MatchConfidence;
  source?: ApplicationSource | null;
  matched_package?: string | null;
  version?: string | null;
  executable_path?: string | null;
  details?: string | null;
}

export interface DiscoverySnapshot {
  is_snap_available: boolean;
  scanned_at_ms: number;
  total_packages_scanned: number;
  results: ProviderDiscoveryResult[];
  diagnostics?: string | null;
}

const isTauriAvailable = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

export const gatewayService = {
  async listProviders(): Promise<ProviderDefinition[]> {
    if (!isTauriAvailable()) {
      return [
        { id: 'chatgpt', name: 'ChatGPT', description: 'OpenAI ChatGPT desktop application', vendor: 'OpenAI', accent_color: '#10a37f', supported_platforms: ['linux', 'windows', 'macos'], candidate_snap_names: ['chatgpt-desktop'], candidate_desktop_ids: ['chatgpt-desktop_chatgpt-desktop.desktop'], candidate_executables: ['/snap/bin/chatgpt-desktop'] },
        { id: 'gemini', name: 'Gemini', description: 'Google Gemini desktop application', vendor: 'Google', accent_color: '#4285f4', supported_platforms: ['linux', 'windows', 'macos'], candidate_snap_names: ['gemini-desktop'], candidate_desktop_ids: ['gemini-desktop_gemini-desktop.desktop'], candidate_executables: ['/snap/bin/gemini-desktop'] },
        { id: 'claude', name: 'Claude', description: 'Anthropic Claude desktop application', vendor: 'Anthropic', accent_color: '#d97706', supported_platforms: ['linux', 'windows', 'macos'], candidate_snap_names: ['claude-desktop'], candidate_desktop_ids: ['claude.desktop'], candidate_executables: ['claude-desktop'] },
        { id: 'grok', name: 'Grok', description: 'xAI Grok desktop application', vendor: 'xAI', accent_color: '#1d9bf0', supported_platforms: ['linux', 'windows', 'macos'], candidate_snap_names: ['grok-desktop'], candidate_desktop_ids: ['grok.desktop'], candidate_executables: ['grok'] },
        { id: 'perplexity', name: 'Perplexity', description: 'Perplexity AI desktop application', vendor: 'Perplexity AI', accent_color: '#20b2aa', supported_platforms: ['linux', 'windows', 'macos'], candidate_snap_names: ['perplexity-desktop'], candidate_desktop_ids: ['perplexity.desktop'], candidate_executables: ['perplexity'] },
        { id: 'kimi', name: 'Kimi', description: 'Moonshot AI Kimi desktop application', vendor: 'Moonshot AI', accent_color: '#7c3aed', supported_platforms: ['linux', 'windows', 'macos'], candidate_snap_names: ['kimi-desktop'], candidate_desktop_ids: ['kimi.desktop'], candidate_executables: ['kimi'] },
      ];
    }
    return invoke<ProviderDefinition[]>('list_providers');
  },

  async getProviderState(providerId: ProviderId): Promise<ManagedApplication> {
    if (!isTauriAvailable()) {
      return {
        provider_id: providerId,
        lifecycle_state: 'not_installed',
        process: null,
        is_active: false,
        last_state_change_ms: Date.now(),
      };
    }
    return invoke<ManagedApplication>('get_provider_state', { providerId });
  },

  async getActiveProvider(): Promise<string | null> {
    if (!isTauriAvailable()) return 'chatgpt';
    return invoke<string | null>('get_active_provider');
  },

  async setActiveProvider(providerId: ProviderId): Promise<void> {
    if (!isTauriAvailable()) return;
    return invoke<void>('set_active_provider', { providerId });
  },

  async getRuntimeSettings(): Promise<RuntimeSettings> {
    if (!isTauriAvailable()) {
      return {
        resource_saver_mode: true,
        confirm_before_switching: true,
        graceful_shutdown_timeout_secs: 5,
        allow_force_termination: false,
        auto_start_on_boot: false,
        metrics_polling_interval_secs: 3,
      };
    }
    return invoke<RuntimeSettings>('get_runtime_settings');
  },

  async updateRuntimeSettings(settings: RuntimeSettings): Promise<RuntimeSettings> {
    if (!isTauriAvailable()) return settings;
    return invoke<RuntimeSettings>('update_runtime_settings', { settings });
  },

  async discoverProvider(providerId: ProviderId): Promise<InstalledApplication | null> {
    if (!isTauriAvailable()) return null;
    return invoke<InstalledApplication | null>('discover_provider', { providerId });
  },

  async discoverAllProviders(): Promise<DiscoverySnapshot> {
    if (!isTauriAvailable()) {
      return {
        is_snap_available: false,
        scanned_at_ms: Date.now(),
        total_packages_scanned: 0,
        results: [
          { provider_id: 'chatgpt', status: 'not_installed', match_confidence: 'none', source: null },
          { provider_id: 'gemini', status: 'not_installed', match_confidence: 'none', source: null },
          { provider_id: 'claude', status: 'not_installed', match_confidence: 'none', source: null },
          { provider_id: 'grok', status: 'not_installed', match_confidence: 'none', source: null },
          { provider_id: 'perplexity', status: 'not_installed', match_confidence: 'none', source: null },
          { provider_id: 'kimi', status: 'not_installed', match_confidence: 'none', source: null },
        ],
        diagnostics: 'Web/Mock environment: native discovery unavailable',
      };
    }
    return invoke<DiscoverySnapshot>('discover_all_providers');
  },

  async launchProvider(providerId: ProviderId): Promise<ManagedApplication> {
    if (!isTauriAvailable()) {
      throw new Error('Launching external desktop applications requires the native Tauri runtime');
    }
    return invoke<ManagedApplication>('launch_provider', { providerId });
  },

  async stopProvider(providerId: ProviderId): Promise<ManagedApplication> {
    if (!isTauriAvailable()) {
      throw new Error('Stopping external desktop applications requires the native Tauri runtime');
    }
    return invoke<ManagedApplication>('stop_provider', { providerId });
  },

  async switchProvider(providerId: ProviderId): Promise<ManagedApplication> {
    if (!isTauriAvailable()) {
      throw new Error('Switching external desktop applications requires the native Tauri runtime');
    }
    return invoke<ManagedApplication>('switch_provider', { providerId });
  },
};
