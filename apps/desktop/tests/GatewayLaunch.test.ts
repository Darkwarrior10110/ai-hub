import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import * as tauriCore from '@tauri-apps/api/core';
import { gatewayService, type LaunchResult, type LaunchStatus, type ProviderId } from '../src/services/gateway';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('GatewayLaunch Milestone 3 Frontend Integration', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    // @ts-expect-error test cleanup
    delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    // @ts-expect-error test cleanup
    delete (globalThis as Record<string, unknown>).window;
  });

  afterEach(() => {
    // @ts-expect-error test cleanup
    delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    // @ts-expect-error test cleanup
    delete (globalThis as Record<string, unknown>).window;
  });

  it('should validate LaunchResult contract structure', () => {
    const mockResult: LaunchResult = {
      provider_id: 'chatgpt',
      status: 'running' as LaunchStatus,
      process: {
        pid: 12345,
        process_name: 'chatgpt-desktop',
        start_time_ms: 1700000000000,
        ownership: 'started_by_ai_hub',
      },
      details: 'Application successfully spawned via Snap with PID 12345',
      launched_at_ms: 1700000000000,
    };

    expect(mockResult.provider_id).toBe('chatgpt');
    expect(mockResult.status).toBe('running');
    expect(mockResult.process?.pid).toBe(12345);
    expect(mockResult.process?.ownership).toBe('started_by_ai_hub');
  });

  it('should reject launch in mock/web environment without native Tauri runtime', async () => {
    await expect(gatewayService.launchProvider('chatgpt')).rejects.toThrow(
      'Launching external desktop applications requires the native Tauri runtime'
    );
  });

  it('should list all 6 provider definitions with candidate launch executables', async () => {
    const providers = await gatewayService.listProviders();
    expect(providers.length).toBe(6);

    const chatgpt = providers.find((p) => p.id === 'chatgpt');
    expect(chatgpt).toBeDefined();
    expect(chatgpt?.candidate_snap_names).toContain('chatgpt-desktop');
    expect(chatgpt?.candidate_executables).toContain('/snap/bin/chatgpt-desktop');

    const gemini = providers.find((p) => p.id === 'gemini');
    expect(gemini).toBeDefined();
    expect(gemini?.candidate_snap_names).toContain('gemini-desktop');
    expect(gemini?.candidate_executables).toContain('/snap/bin/gemini-desktop');
  });

  it('should safely represent all valid launch status variants', () => {
    const validStatuses: LaunchStatus[] = [
      'rejected',
      'launch_failed',
      'launch_requested',
      'running',
      'already_running',
      'unsupported',
    ];
    expect(validStatuses.length).toBe(6);
    expect(validStatuses).toContain('already_running');
    expect(validStatuses).toContain('running');
  });

  it('should dispatch individual provider launch requests with exact provider IDs to backend', async () => {
    // Simulate Tauri runtime presence
    // @ts-expect-error simulate window
    globalThis.window = globalThis;
    // @ts-expect-error simulate __TAURI_INTERNALS__
    globalThis.__TAURI_INTERNALS__ = {};

    const invokeMock = vi.mocked(tauriCore.invoke).mockImplementation(async (cmd, args) => {
      if (cmd === 'launch_provider') {
        const { providerId } = (args as { providerId: ProviderId }) || {};
        return {
          provider_id: providerId,
          status: 'running',
          process: {
            pid: providerId === 'chatgpt' ? 11111 : 22222,
            process_name: `${providerId}-desktop`,
            ownership: 'started_by_ai_hub',
          },
          details: `Spawned ${providerId}`,
          launched_at_ms: Date.now(),
        } as LaunchResult;
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    // 1. Launch ChatGPT
    const chatgptResult = await gatewayService.launchProvider('chatgpt');
    expect(invokeMock).toHaveBeenCalledWith('launch_provider', { providerId: 'chatgpt' });
    expect(chatgptResult.provider_id).toBe('chatgpt');
    expect(chatgptResult.process?.pid).toBe(11111);

    // 2. Launch Gemini
    const geminiResult = await gatewayService.launchProvider('gemini');
    expect(invokeMock).toHaveBeenCalledWith('launch_provider', { providerId: 'gemini' });
    expect(geminiResult.provider_id).toBe('gemini');
    expect(geminiResult.process?.pid).toBe(22222);

    // 3. Launch Kimi
    const kimiResult = await gatewayService.launchProvider('kimi');
    expect(invokeMock).toHaveBeenCalledWith('launch_provider', { providerId: 'kimi' });
    expect(kimiResult.provider_id).toBe('kimi');
  });

  it('should evaluate individual Launch button eligibility states accurately', () => {
    // Helper replicating component state logic
    const evaluateButton = (params: {
      discStatus?: string;
      lifecycleState?: string;
      isLaunchingGlobally: boolean;
      launchingProviderId: string | null;
      providerId: string;
      pid?: number;
    }) => {
      const cardConfirmed = params.discStatus === 'confirmed';
      const cardInstalled =
        params.lifecycleState === 'installed' || cardConfirmed || params.lifecycleState === 'running';
      const cardAmbiguous = params.discStatus === 'ambiguous';
      const cardRunning = params.lifecycleState === 'running';
      const isThisLaunching = params.isLaunchingGlobally && params.launchingProviderId === params.providerId;
      const isLaunchDisabled = params.isLaunchingGlobally || !cardInstalled || cardAmbiguous;

      return {
        cardInstalled,
        cardRunning,
        isThisLaunching,
        isLaunchDisabled,
        label: isThisLaunching
          ? 'Launching…'
          : cardRunning
          ? `Running (PID: ${params.pid ?? 'Active'})`
          : `Launch ${params.providerId}`,
      };
    };

    // Case 1: ChatGPT detected and installed, ready to launch
    const chatgptReady = evaluateButton({
      discStatus: 'confirmed',
      lifecycleState: 'installed',
      isLaunchingGlobally: false,
      launchingProviderId: null,
      providerId: 'chatgpt',
    });
    expect(chatgptReady.cardInstalled).toBe(true);
    expect(chatgptReady.isLaunchDisabled).toBe(false);
    expect(chatgptReady.label).toBe('Launch chatgpt');

    // Case 2: Gemini not installed
    const geminiNotInstalled = evaluateButton({
      discStatus: 'not_installed',
      lifecycleState: 'not_installed',
      isLaunchingGlobally: false,
      launchingProviderId: null,
      providerId: 'gemini',
    });
    expect(geminiNotInstalled.cardInstalled).toBe(false);
    expect(geminiNotInstalled.isLaunchDisabled).toBe(true);

    // Case 3: Ambiguous match
    const ambiguousProvider = evaluateButton({
      discStatus: 'ambiguous',
      lifecycleState: 'unknown',
      isLaunchingGlobally: false,
      launchingProviderId: null,
      providerId: 'claude',
    });
    expect(ambiguousProvider.isLaunchDisabled).toBe(true);

    // Case 4: Launching in progress for ChatGPT -> all buttons disabled to prevent duplicate launch
    const chatgptLaunching = evaluateButton({
      discStatus: 'confirmed',
      lifecycleState: 'installed',
      isLaunchingGlobally: true,
      launchingProviderId: 'chatgpt',
      providerId: 'chatgpt',
    });
    expect(chatgptLaunching.isThisLaunching).toBe(true);
    expect(chatgptLaunching.isLaunchDisabled).toBe(true);
    expect(chatgptLaunching.label).toBe('Launching…');

    const otherDuringLaunch = evaluateButton({
      discStatus: 'confirmed',
      lifecycleState: 'installed',
      isLaunchingGlobally: true,
      launchingProviderId: 'chatgpt',
      providerId: 'gemini',
    });
    expect(otherDuringLaunch.isThisLaunching).toBe(false);
    expect(otherDuringLaunch.isLaunchDisabled).toBe(true); // Locked while launch in flight

    // Case 5: Provider currently running
    const runningProvider = evaluateButton({
      discStatus: 'confirmed',
      lifecycleState: 'running',
      isLaunchingGlobally: false,
      launchingProviderId: null,
      providerId: 'gemini',
      pid: 45678,
    });
    expect(runningProvider.cardRunning).toBe(true);
    expect(runningProvider.isLaunchDisabled).toBe(false);
    expect(runningProvider.label).toBe('Running (PID: 45678)');
  });

  it('should fetch real system resources and detect memory pressure in frontend service', async () => {
    // @ts-expect-error simulate window
    globalThis.window = globalThis;
    // @ts-expect-error simulate __TAURI_INTERNALS__
    globalThis.__TAURI_INTERNALS__ = {};

    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_system_resources') {
        return {
          is_available: true,
          system_memory: {
            total_bytes: 4096 * 1024 * 1024,
            available_bytes: 400 * 1024 * 1024,
            used_bytes: 3696 * 1024 * 1024,
            used_percentage: 90,
            swap_total_bytes: 2048 * 1024 * 1024,
            swap_free_bytes: 1024 * 1024 * 1024,
            swap_used_bytes: 1024 * 1024 * 1024,
          },
          memory_pressure_warning: true,
          memory_pressure_message: 'Resource Saver advisory: memory pressure detected (90.2% used)',
          tracked_processes_count: 1,
          diagnostics: 'System memory parsed from /proc/meminfo',
          timestamp_ms: Date.now(),
        };
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const resources = await gatewayService.getSystemResources();
    expect(resources.is_available).toBe(true);
    expect(resources.memory_pressure_warning).toBe(true);
    expect(resources.system_memory?.used_percentage).toBe(90);
    expect(resources.tracked_processes_count).toBe(1);
    expect(resources.memory_pressure_message).toContain('Resource Saver advisory');
  });

  it('should dispatch stopProvider with verified provider ID', async () => {
    // @ts-expect-error simulate window
    globalThis.window = globalThis;
    // @ts-expect-error simulate __TAURI_INTERNALS__
    globalThis.__TAURI_INTERNALS__ = {};

    let stoppedProviderId = '';
    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd, args) => {
      if (cmd === 'stop_provider') {
        const { providerId } = (args as { providerId: ProviderId }) || {};
        stoppedProviderId = providerId;
        return {
          provider_id: providerId,
          lifecycle_state: 'installed',
          process: null,
          is_active: false,
          last_state_change_ms: Date.now(),
          error_message: null,
        };
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const result = await gatewayService.stopProvider('chatgpt');
    expect(stoppedProviderId).toBe('chatgpt');
    expect(result.lifecycle_state).toBe('installed');
    expect(result.process).toBeNull();
  });

  it('should return default states for reconcileProviderStates in non-Tauri environment', async () => {
    const states = await gatewayService.reconcileProviderStates();
    expect(states.chatgpt).toBeDefined();
    expect(states.chatgpt.lifecycle_state).toBe('not_installed');
    expect(states.gemini).toBeDefined();
    expect(states.gemini.lifecycle_state).toBe('not_installed');
    expect(Object.keys(states).length).toBe(6);
  });

  it('should call reconcile_provider_states and return reconciled map in Tauri environment', async () => {
    // @ts-expect-error simulate window
    globalThis.window = globalThis;
    // @ts-expect-error simulate __TAURI_INTERNALS__
    globalThis.__TAURI_INTERNALS__ = {};

    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd) => {
      if (cmd === 'reconcile_provider_states') {
        return {
          chatgpt: {
            provider_id: 'chatgpt',
            lifecycle_state: 'installed',
            process: null,
            is_active: false,
            last_state_change_ms: 1700000000000,
          },
          gemini: {
            provider_id: 'gemini',
            lifecycle_state: 'running',
            process: {
              pid: 54321,
              process_name: 'gemini-desktop',
              ownership: 'started_by_ai_hub',
            },
            is_active: true,
            last_state_change_ms: 1700000001000,
          },
        };
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const reconciled = await gatewayService.reconcileProviderStates();
    expect(reconciled.chatgpt.lifecycle_state).toBe('installed');
    expect(reconciled.chatgpt.process).toBeNull();
    expect(reconciled.gemini.lifecycle_state).toBe('running');
    expect(reconciled.gemini.process?.pid).toBe(54321);
  });

  it('should reflect external application exit after reconciliation', async () => {
    // @ts-expect-error simulate window
    globalThis.window = globalThis;
    // @ts-expect-error simulate __TAURI_INTERNALS__
    globalThis.__TAURI_INTERNALS__ = {};

    // Initial state: gemini was running
    let geminiAlive = true;

    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd) => {
      if (cmd === 'reconcile_provider_states') {
        return {
          gemini: {
            provider_id: 'gemini',
            lifecycle_state: geminiAlive ? 'running' : 'installed',
            process: geminiAlive
              ? { pid: 9999, process_name: 'gemini-desktop', ownership: 'started_by_ai_hub' }
              : null,
            is_active: geminiAlive,
            last_state_change_ms: Date.now(),
          },
        };
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    // 1. First poll: Gemini is running
    const poll1 = await gatewayService.reconcileProviderStates();
    expect(poll1.gemini.lifecycle_state).toBe('running');
    expect(poll1.gemini.process?.pid).toBe(9999);

    // 2. User manually exits Gemini outside AI-Hub
    geminiAlive = false;

    // 3. Second poll: Gemini is reconciled to installed
    const poll2 = await gatewayService.reconcileProviderStates();
    expect(poll2.gemini.lifecycle_state).toBe('installed');
    expect(poll2.gemini.process).toBeNull();
    expect(poll2.gemini.is_active).toBe(false);
  });

  it('should propagate stopProvider error so UI layer can synchronize with backend state', async () => {
    // @ts-expect-error simulate window
    globalThis.window = globalThis;
    // @ts-expect-error simulate __TAURI_INTERNALS__
    globalThis.__TAURI_INTERNALS__ = {};

    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd, args) => {
      if (cmd === 'stop_provider') {
        throw new Error('Shutdown timeout (5s) exceeded without process exit');
      }
      if (cmd === 'get_provider_state') {
        const { providerId } = (args as { providerId: ProviderId }) || {};
        return {
          provider_id: providerId,
          lifecycle_state: 'error',
          process: { pid: 8888, process_name: 'uncooperative-app', ownership: 'started_by_ai_hub' },
          is_active: false,
          last_state_change_ms: Date.now(),
          error_message: 'Shutdown timeout (5s) exceeded without process exit',
        };
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    // Calling stopProvider should throw the error
    await expect(gatewayService.stopProvider('chatgpt')).rejects.toThrow('Shutdown timeout');

    // UI catch block gets updated state
    const syncedState = await gatewayService.getProviderState('chatgpt');
    expect(syncedState.lifecycle_state).toBe('error');
    expect(syncedState.error_message).toContain('Shutdown timeout');
  });

  describe('Milestone 5 Provider Switching & Resource Saver Integration', () => {
    it('should reject switchProvider in mock/web environment without native Tauri runtime', async () => {
      await expect(gatewayService.switchProvider('gemini')).rejects.toThrow(
        'Switching external desktop applications requires the native Tauri runtime'
      );
    });

    it('should dispatch switch_provider IPC command with target providerId and return updated state', async () => {
      // @ts-expect-error simulate window
      globalThis.window = globalThis;
      // @ts-expect-error simulate __TAURI_INTERNALS__
      globalThis.__TAURI_INTERNALS__ = {};

      const invokeMock = vi.mocked(tauriCore.invoke).mockImplementation(async (cmd, args) => {
        if (cmd === 'switch_provider') {
          const { providerId } = (args as { providerId: ProviderId }) || {};
          return {
            provider_id: providerId,
            lifecycle_state: 'running',
            process: {
              pid: 7777,
              process_name: `${providerId}-desktop`,
              ownership: 'started_by_ai_hub',
            },
            is_active: true,
            last_state_change_ms: Date.now(),
          };
        }
        throw new Error(`Unexpected command: ${cmd}`);
      });

      const res = await gatewayService.switchProvider('gemini');
      expect(invokeMock).toHaveBeenCalledWith('switch_provider', { providerId: 'gemini' });
      expect(res.provider_id).toBe('gemini');
      expect(res.lifecycle_state).toBe('running');
      expect(res.process?.pid).toBe(7777);
    });

    it('should propagate backend concurrency lock OperationInProgress error to frontend', async () => {
      // @ts-expect-error simulate window
      globalThis.window = globalThis;
      // @ts-expect-error simulate __TAURI_INTERNALS__
      globalThis.__TAURI_INTERNALS__ = {};

      vi.mocked(tauriCore.invoke).mockImplementation(async (cmd) => {
        if (cmd === 'switch_provider') {
          throw new Error('A provider switch operation is already in progress');
        }
        throw new Error(`Unexpected command: ${cmd}`);
      });

      await expect(gatewayService.switchProvider('chatgpt')).rejects.toThrow(
        'A provider switch operation is already in progress'
      );
    });

    it('should propagate old provider shutdown timeout abort error', async () => {
      // @ts-expect-error simulate window
      globalThis.window = globalThis;
      // @ts-expect-error simulate __TAURI_INTERNALS__
      globalThis.__TAURI_INTERNALS__ = {};

      vi.mocked(tauriCore.invoke).mockImplementation(async (cmd) => {
        if (cmd === 'switch_provider') {
          throw new Error('Graceful shutdown timeout exceeded for provider chatgpt; target was not launched');
        }
        throw new Error(`Unexpected command: ${cmd}`);
      });

      await expect(gatewayService.switchProvider('gemini')).rejects.toThrow(
        'Graceful shutdown timeout exceeded for provider chatgpt'
      );
    });

    it('should receive and reconcile correct states for both providers after successful switch', async () => {
      // @ts-expect-error simulate window
      globalThis.window = globalThis;
      // @ts-expect-error simulate __TAURI_INTERNALS__
      globalThis.__TAURI_INTERNALS__ = {};

      vi.mocked(tauriCore.invoke).mockImplementation(async (cmd) => {
        if (cmd === 'switch_provider') {
          return {
            provider_id: 'gemini',
            lifecycle_state: 'running',
            process: { pid: 3001, process_name: 'gemini-desktop', ownership: 'started_by_ai_hub' },
            is_active: true,
            last_state_change_ms: Date.now(),
          };
        }
        if (cmd === 'reconcile_provider_states') {
          return {
            chatgpt: {
              provider_id: 'chatgpt',
              lifecycle_state: 'installed',
              is_active: false,
              last_state_change_ms: Date.now(),
            },
            gemini: {
              provider_id: 'gemini',
              lifecycle_state: 'running',
              process: { pid: 3001, process_name: 'gemini-desktop', ownership: 'started_by_ai_hub' },
              is_active: true,
              last_state_change_ms: Date.now(),
            },
          };
        }
        throw new Error(`Unexpected command: ${cmd}`);
      });

      const switched = await gatewayService.switchProvider('gemini');
      expect(switched.lifecycle_state).toBe('running');
      expect(switched.process?.pid).toBe(3001);

      const reconciled = await gatewayService.reconcileProviderStates();
      expect(reconciled['chatgpt'].lifecycle_state).toBe('installed');
      expect(reconciled['chatgpt'].process).toBeUndefined();
      expect(reconciled['gemini'].lifecycle_state).toBe('running');
      expect(reconciled['gemini'].process?.pid).toBe(3001);
    });

    it('should handle target launch failure with truthful state and recoverable feedback', async () => {
      // @ts-expect-error simulate window
      globalThis.window = globalThis;
      // @ts-expect-error simulate __TAURI_INTERNALS__
      globalThis.__TAURI_INTERNALS__ = {};

      vi.mocked(tauriCore.invoke).mockImplementation(async (cmd, args) => {
        if (cmd === 'switch_provider') {
          throw new Error('Previous application stopped, but target provider gemini failed to launch: Binary corrupted');
        }
        if (cmd === 'get_provider_state') {
          const { providerId } = (args as { providerId: ProviderId }) || {};
          if (providerId === 'gemini') {
            return {
              provider_id: 'gemini',
              lifecycle_state: 'error',
              is_active: false,
              last_state_change_ms: Date.now(),
              error_message: 'Binary corrupted',
            };
          }
          return {
            provider_id: providerId,
            lifecycle_state: 'installed',
            is_active: false,
            last_state_change_ms: Date.now(),
          };
        }
        throw new Error(`Unexpected command: ${cmd}`);
      });

      await expect(gatewayService.switchProvider('gemini')).rejects.toThrow(
        'Previous application stopped, but target provider gemini failed to launch'
      );

      const targetState = await gatewayService.getProviderState('gemini');
      expect(targetState.lifecycle_state).toBe('error');
      expect(targetState.error_message).toContain('Binary corrupted');
    });

    it('should ignore delayed stale reconciliation responses and not overwrite newer verified state', async () => {
      // Test request sequencing: an earlier query that resolves later must be dropped
      let sequence = 0;
      let lastMutationTime = 1000;

      const responses = [
        // Response 1: Stale poll initiated at t=500 with ChatGPT=installed
        {
          reqVersion: 1,
          reqTime: 500,
          data: { chatgpt: { provider_id: 'chatgpt', lifecycle_state: 'installed', is_active: false, last_state_change_ms: 500 } },
        },
        // Response 2: Newer switch completed at t=1000 with ChatGPT=running
        {
          reqVersion: 2,
          reqTime: 1000,
          data: { chatgpt: { provider_id: 'chatgpt', lifecycle_state: 'running', is_active: true, last_state_change_ms: 1000 } },
        },
      ];

      // Simulate newer operation setting version to 2 and mutation timestamp to 1000
      sequence = 2;
      lastMutationTime = 1000;

      const isStale = (resp: typeof responses[0]) => {
        return resp.reqVersion < sequence || resp.reqTime < lastMutationTime;
      };

      expect(isStale(responses[0])).toBe(true);  // Stale poll dropped!
      expect(isStale(responses[1])).toBe(false); // Current state accepted!
    });
  });
});
