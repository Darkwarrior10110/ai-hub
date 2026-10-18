import { useState } from 'react';
import { useUI } from '../../providers/UIProvider';
import { PROVIDERS, getProviderInitial } from '../../types';
import type { ProviderId } from '../../services/gateway';
import {
  Sparkles,
  Play,
  Square,
  ShieldCheck,
  Cpu,
  AlertCircle,
  RefreshCw,
  Layers,
  CheckCircle2,
  XCircle,
  Info,
  ArrowRightLeft,
} from 'lucide-react';
import './GatewayDashboard.css';

export function GatewayDashboard() {
  const {
    selectedProvider,
    setSelectedProvider,
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
    updateSettings,
  } = useUI();

  const providerMeta = PROVIDERS.find((p) => p.id === selectedProvider) || PROVIDERS[0];
  const appState = managedApps[selectedProvider];
  const discoveryResult = discoverySnapshot?.results.find((r) => r.provider_id === selectedProvider);
  const [actionError, setActionError] = useState<string | null>(null);
  const [pendingSwitchProviderId, setPendingSwitchProviderId] = useState<ProviderId | null>(null);

  // Compute active running providers
  const runningProviders = PROVIDERS.filter((p) => managedApps[p.id]?.lifecycle_state === 'running');
  const otherRunningProviders = runningProviders.filter((p) => p.id !== selectedProvider);
  const isAnyStopping = Object.values(isStopping).some(Boolean);
  const isBusy = isLaunching || isSwitching || isAnyStopping || isScanning;

  const handleLaunch = async () => {
    setActionError(null);
    clearSwitchError();

    // In Enforced Resource Saver mode, if another provider is running, route through switch flow
    if (runtimeSettings.resource_saver_mode && otherRunningProviders.length > 0) {
      handleSwitchRequest(selectedProvider);
      return;
    }

    try {
      await launchProvider(selectedProvider);
    } catch (err: unknown) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleCardLaunch = async (e: React.MouseEvent, providerId: ProviderId) => {
    e.stopPropagation();
    setSelectedProvider(providerId);
    setActionError(null);
    clearSwitchError();

    // In Enforced Resource Saver mode, if another provider is running, route through switch flow
    const otherRunning = runningProviders.filter((p) => p.id !== providerId);
    if (runtimeSettings.resource_saver_mode && otherRunning.length > 0) {
      handleSwitchRequest(providerId);
      return;
    }

    try {
      await launchProvider(providerId);
    } catch (err: unknown) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleSwitchRequest = (targetId: ProviderId) => {
    setActionError(null);
    clearSwitchError();
    const otherRunning = runningProviders.filter((p) => p.id !== targetId);
    if (
      runtimeSettings.confirm_before_switching &&
      runtimeSettings.resource_saver_mode &&
      otherRunning.length > 0
    ) {
      setPendingSwitchProviderId(targetId);
    } else {
      executeSwitch(targetId);
    }
  };

  const executeSwitch = async (targetId: ProviderId) => {
    setPendingSwitchProviderId(null);
    setActionError(null);
    clearSwitchError();
    try {
      await switchProvider(targetId);
    } catch (err: unknown) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleToggleResourceSaver = async () => {
    setActionError(null);
    try {
      await updateSettings({
        ...runtimeSettings,
        resource_saver_mode: !runtimeSettings.resource_saver_mode,
      });
    } catch (err: unknown) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleStop = async () => {
    setActionError(null);
    clearSwitchError();
    try {
      await stopProvider(selectedProvider);
    } catch (err: unknown) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleCardStop = async (e: React.MouseEvent, providerId: ProviderId) => {
    e.stopPropagation();
    setActionError(null);
    clearSwitchError();
    try {
      await stopProvider(providerId);
    } catch (err: unknown) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleScan = async () => {
    setActionError(null);
    clearSwitchError();
    await scanApplications();
  };

  const isConfirmed = discoveryResult?.status === 'confirmed';
  const isInstalled = appState?.lifecycle_state === 'installed' || isConfirmed;
  const isRunning = appState?.lifecycle_state === 'running';
  const displayError = actionError || switchError;

  return (
    <div className="gateway-dashboard">
      {/* Top Header */}
      <header className="gateway-header">
        <div className="header-provider-badge" style={{ backgroundColor: providerMeta.color }}>
          <Sparkles size={16} />
        </div>
        <div className="header-info">
          <div className="header-title-row">
            <h1 className="header-provider-name">{providerMeta.name}</h1>
            <span className="vendor-tag">{providerMeta.description}</span>
            {discoveryResult?.version && (
              <span className="vendor-tag" title={`Package version: ${discoveryResult.version}`}>
                v{discoveryResult.version}
              </span>
            )}
          </div>
          <div className="header-status-row">
            {discoveryResult?.status === 'confirmed' ? (
              <span className="status-pill installed">
                <span className="status-indicator-dot" />
                Detected (Snap)
              </span>
            ) : discoveryResult?.status === 'ambiguous' ? (
              <span className="status-pill ambiguous" title={discoveryResult.details || undefined}>
                <span className="status-indicator-dot" />
                Needs Review
              </span>
            ) : discoveryResult?.status === 'unavailable' ? (
              <span className="status-pill unavailable" title="Snap tooling unavailable">
                <span className="status-indicator-dot" />
                Snap Unavailable
              </span>
            ) : (
              <span className="status-pill not_installed">
                <span className="status-indicator-dot" />
                Not Installed
              </span>
            )}

            {isRunning && (
              <span className="status-pill running">
                <span className="status-indicator-dot" />
                Running
              </span>
            )}

            <button
              type="button"
              className={`resource-saver-pill toggleable ${runtimeSettings.resource_saver_mode ? 'enforced' : 'advisory'}`}
              onClick={handleToggleResourceSaver}
              title={
                runtimeSettings.resource_saver_mode
                  ? 'Resource Saver Mode: Single-Instance Enforced. Click to set Advisory Only.'
                  : 'Resource Saver Mode: Advisory Only. Click to enforce Single-Instance.'
              }
            >
              <ShieldCheck size={13} />
              <span>
                {runtimeSettings.resource_saver_mode
                  ? 'Resource Saver: Enforced'
                  : 'Resource Saver: Advisory Only'}
              </span>
            </button>
          </div>
        </div>

        <div className="header-actions">
          <button
            className="refresh-btn"
            onClick={handleScan}
            disabled={isScanning || isBusy}
            title={discoverySnapshot ? `Scanned ${discoverySnapshot.total_packages_scanned} packages` : 'Scan installed applications'}
          >
            <RefreshCw size={15} className={isScanning ? 'spinning' : ''} />
            <span>{isScanning ? 'Scanning…' : 'Scan System'}</span>
          </button>
        </div>
      </header>

      {/* Main Content Area */}
      <div className="gateway-content">
        {/* Error notification banner if any action failed */}
        {displayError && (
          <div className="gateway-alert-banner">
            <AlertCircle size={18} />
            <div className="alert-content">
              <strong>Application Lifecycle Notice</strong>
              <p>{displayError}</p>
            </div>
            <button
              className="alert-close"
              onClick={() => {
                setActionError(null);
                clearSwitchError();
              }}
            >
              ×
            </button>
          </div>
        )}

        {/* Switching In-Flight Banner */}
        {isSwitching && (
          <div className="gateway-switching-banner">
            <RefreshCw size={16} className="spinning" />
            <div>
              <strong>Switching Applications…</strong>
              <p>
                {switchingTargetId
                  ? `Gracefully closing active provider and launching ${PROVIDERS.find((p) => p.id === switchingTargetId)?.name || switchingTargetId} to protect system memory.`
                  : 'Coordinating safe process switch and resource conservation.'}
              </p>
            </div>
          </div>
        )}

        {/* Primary Control Card */}
        <section className="control-hero-card">
          <div className="hero-glow" style={{ background: `radial-gradient(circle, ${providerMeta.color}18 0%, transparent 70%)` }} />
          
          <div className="hero-content">
            <div className="hero-meta">
              <span className="hero-category">Managed AI Desktop Application</span>
              <h2 className="hero-title">{providerMeta.name} Control Center</h2>
              <p className="hero-description">
                AI-Hub coordinates external application launching, safe process tracking, and single-instance resource saving without requiring in-app browser overhead.
              </p>
            </div>

            <div className="hero-actions">
              {isInstalled ? (
                <>
                  <div className="discovery-meta-row">
                    <span className="discovery-meta-badge">
                      Package: <strong>{discoveryResult?.matched_package || providerMeta.id}</strong>
                    </span>
                    {discoveryResult?.version && (
                      <span className="discovery-meta-badge">
                        Version: <strong>{discoveryResult.version}</strong>
                      </span>
                    )}
                    <span className="discovery-meta-badge">
                      Source: <strong>{discoveryResult?.source || 'Snap Package'}</strong>
                    </span>
                  </div>
                  <div style={{ display: 'flex', gap: '10px', marginTop: '12px' }}>
                    {!isRunning && runtimeSettings.resource_saver_mode && otherRunningProviders.length > 0 ? (
                      <button
                        className="launch-btn switch-primary"
                        onClick={() => handleSwitchRequest(selectedProvider)}
                        disabled={isBusy || !isInstalled || discoveryResult?.status === 'ambiguous'}
                      >
                        {isSwitching && switchingTargetId === selectedProvider ? (
                          <>
                            <RefreshCw size={15} className="spinning" />
                            <span>Switching to {providerMeta.name}…</span>
                          </>
                        ) : (
                          <>
                            <ArrowRightLeft size={16} />
                            <span>Switch to {providerMeta.name}</span>
                          </>
                        )}
                      </button>
                    ) : (
                      <button
                        className="launch-btn primary"
                        onClick={handleLaunch}
                        disabled={isBusy || !isInstalled || discoveryResult?.status === 'ambiguous'}
                        style={{ backgroundColor: providerMeta.color }}
                      >
                        {isLaunching && launchingProviderId === selectedProvider ? (
                          <>
                            <RefreshCw size={15} className="spinning" />
                            <span>Launching {providerMeta.name}…</span>
                          </>
                        ) : (
                          <>
                            <Play size={16} fill="currentColor" />
                            <span>
                              {isRunning
                                ? `Launch Again (PID: ${appState?.process?.pid ?? 'Active'})`
                                : `Launch ${providerMeta.name}`}
                            </span>
                          </>
                        )}
                      </button>
                    )}
                    <button
                      className="stop-btn secondary"
                      onClick={handleStop}
                      disabled={!isRunning || isBusy}
                    >
                      {isStopping[selectedProvider] ? (
                        <>
                          <RefreshCw size={15} className="spinning" />
                          <span>Stopping…</span>
                        </>
                      ) : (
                        <>
                          <Square size={15} />
                          <span>Stop Process</span>
                        </>
                      )}
                    </button>
                  </div>

                  {isRunning && appState?.process && (
                    <div className="discovery-meta-row" style={{ marginTop: '8px' }}>
                      <span className="discovery-meta-badge">
                        PID: <strong>{appState.process.pid}</strong>
                      </span>
                      {appState.process.start_time_ticks != null && (
                        <span className="discovery-meta-badge">
                          Start Ticks: <strong>{appState.process.start_time_ticks}</strong>
                        </span>
                      )}
                      <span className="discovery-meta-badge">
                        Ownership: <strong>{appState.process.ownership === 'started_by_ai_hub' ? 'AI-Hub Managed' : 'External'}</strong>
                      </span>
                    </div>
                  )}

                  {lastLaunchResult && lastLaunchResult.provider_id === selectedProvider && (
                    <div className={`launch-feedback-banner ${lastLaunchResult.status}`}>
                      {lastLaunchResult.status === 'running' ? (
                        <CheckCircle2 size={16} />
                      ) : lastLaunchResult.status === 'already_running' ? (
                        <Info size={16} />
                      ) : (
                        <AlertCircle size={16} />
                      )}
                      <div>
                        <div className="launch-feedback-title">
                          {lastLaunchResult.status === 'running'
                            ? `Application Running (PID: ${lastLaunchResult.process?.pid})`
                            : lastLaunchResult.status === 'already_running'
                            ? `Application Already Running (PID: ${lastLaunchResult.process?.pid})`
                            : lastLaunchResult.status === 'launch_requested'
                            ? 'Launch Requested'
                            : `Launch ${lastLaunchResult.status}`}
                        </div>
                        {lastLaunchResult.details && (
                          <div className="launch-feedback-desc">{lastLaunchResult.details}</div>
                        )}
                      </div>
                    </div>
                  )}

                  <p className="milestone-notice">
                    ✓ Milestone 5.1 Active: Safe Provider Switching & Enforced-Mode Safety.
                  </p>
                </>
              ) : discoveryResult?.status === 'ambiguous' ? (
                <div className="not-installed-box">
                  <AlertCircle size={16} />
                  <span>
                    Ambiguous match found for {providerMeta.name}: {discoveryResult.matched_package}. {discoveryResult.details}
                  </span>
                </div>
              ) : (
                <div className="not-installed-box">
                  <Info size={16} />
                  <span>
                    No native desktop application or Snap package detected for {providerMeta.name} on this system.
                  </span>
                </div>
              )}
            </div>
          </div>
        </section>

        {/* Memory Pressure Advisory Banner */}
        {systemResources?.memory_pressure_warning && (
          <div className="gateway-resource-warning-banner">
            <AlertCircle size={18} />
            <div className="alert-content">
              <strong>Resource Saver Advisory</strong>
              <p>
                {systemResources.memory_pressure_message ||
                  'System memory pressure detected. Applications may run slower under heavy memory load.'}
              </p>
            </div>
          </div>
        )}

        {/* System Monitoring & Resource Guard Grid */}
        <section className="metrics-grid">
          <div className="metric-card">
            <div className="metric-icon">
              <Cpu size={18} />
            </div>
            <div className="metric-details">
              <span className="metric-label">System Memory (/proc/meminfo)</span>
              <span className="metric-value">
                {systemResources?.system_memory
                  ? `${systemResources.system_memory.used_percentage}% Used (${(systemResources.system_memory.available_bytes / (1024 * 1024 * 1024)).toFixed(1)} GB Free)`
                  : 'Monitoring active'}
              </span>
              <span className="metric-hint">
                {systemResources?.system_memory
                  ? `RAM: ${(systemResources.system_memory.used_bytes / (1024 * 1024 * 1024)).toFixed(1)} / ${(systemResources.system_memory.total_bytes / (1024 * 1024 * 1024)).toFixed(1)} GB • Swap: ${(systemResources.system_memory.swap_used_bytes / (1024 * 1024)).toFixed(0)} MB`
                  : 'Parsing /proc/meminfo'}
              </span>
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">
              <ShieldCheck size={18} />
            </div>
            <div className="metric-details">
              <span className="metric-label">Process Safety Policy</span>
              <span className="metric-value">Graceful ({runtimeSettings.graceful_shutdown_timeout_secs}s timeout)</span>
              <span className="metric-hint">Verified PID/UID before SIGTERM. Zero dangerous killall commands</span>
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">
              <Layers size={18} />
            </div>
            <div className="metric-details">
              <span className="metric-label">Tracked Processes</span>
              <span className="metric-value">
                {appState?.process
                  ? `PID: ${appState.process.pid}`
                  : `${systemResources?.tracked_processes_count ?? 0} active`}
              </span>
              <span className="metric-hint">
                {appState?.process
                  ? `Ticks: ${appState.process.start_time_ticks ?? 'verified'} • AI-Hub Managed`
                  : 'Real-time OS process monitoring'}
              </span>
            </div>
          </div>
        </section>

        {/* Application Catalog Fleet Matrix */}
        <section className="fleet-section">
          <div className="fleet-header">
            <h3>Provider Fleet Status</h3>
            <span className="fleet-subtitle">Real operating system discovery across Linux Snap & Desktop environments</span>
          </div>

          <div className="fleet-grid">
            {PROVIDERS.map((p) => {
              const disc = discoverySnapshot?.results.find((r) => r.provider_id === p.id);
              const isSelected = selectedProvider === p.id;
              const pState = managedApps[p.id];
              const cardConfirmed = disc?.status === 'confirmed';
              const cardInstalled = pState?.lifecycle_state === 'installed' || cardConfirmed || pState?.lifecycle_state === 'running';
              const cardAmbiguous = disc?.status === 'ambiguous';
              const cardRunning = pState?.lifecycle_state === 'running';
              const isThisLaunching = isLaunching && launchingProviderId === p.id;
              const isThisSwitching = isSwitching && switchingTargetId === p.id;
              const isOldStopping = isSwitching && cardRunning && switchingTargetId !== p.id;
              const otherRunningForCard = runningProviders.filter((rp) => rp.id !== p.id);
              const isSwitchAction = !cardRunning && runtimeSettings.resource_saver_mode && otherRunningForCard.length > 0;
              const isLaunchDisabled = isBusy || !cardInstalled || cardAmbiguous;

              return (
                <div
                  key={p.id}
                  className={`fleet-card ${isSelected ? 'selected' : ''}`}
                  onClick={() => setSelectedProvider(p.id)}
                >
                  <div className="fleet-card-top">
                    <div
                      className="fleet-avatar"
                      style={{
                        backgroundColor: isSelected ? p.color : 'var(--bg-tertiary)',
                        color: isSelected ? '#fff' : p.color,
                        borderColor: p.color,
                      }}
                    >
                      {getProviderInitial(p.id)}
                    </div>
                    <div className="fleet-name-wrap">
                      <span className="fleet-name">{p.name}</span>
                      <span className="fleet-vendor">
                        {disc?.version ? `v${disc.version} • Snap` : p.description}
                      </span>
                    </div>
                    <div className="fleet-badge-wrap">
                      {disc?.status === 'confirmed' ? (
                        <span className="fleet-installed-tag" title={`Confirmed Snap: ${disc.matched_package}`}>
                          <CheckCircle2 size={13} />
                          <span>Detected</span>
                        </span>
                      ) : disc?.status === 'ambiguous' ? (
                        <span className="fleet-ambiguous-tag" title={disc.details || 'Ambiguous package name'}>
                          <AlertCircle size={13} />
                          <span>Needs Review</span>
                        </span>
                      ) : disc?.status === 'unavailable' ? (
                        <span className="fleet-unavailable-tag" title="Snap tooling unavailable">
                          <Info size={13} />
                          <span>Unavailable</span>
                        </span>
                      ) : (
                        <span className="fleet-missing-tag" title="Not found on system">
                          <XCircle size={13} />
                          <span>Not Installed</span>
                        </span>
                      )}
                    </div>
                  </div>

                  <div className="fleet-card-bottom">
                    <div className="fleet-actions-row">
                      {isSwitchAction ? (
                        <button
                          type="button"
                          className="fleet-launch-btn fleet-switch-btn installed"
                          onClick={(e) => {
                            e.stopPropagation();
                            setSelectedProvider(p.id);
                            handleSwitchRequest(p.id);
                          }}
                          disabled={isLaunchDisabled}
                          data-testid={`switch-btn-${p.id}`}
                          title={
                            isThisSwitching
                              ? `Switching to ${p.name}…`
                              : `Switch to ${p.name} (stops running application)`
                          }
                        >
                          {isThisSwitching ? (
                            <>
                              <RefreshCw size={13} className="spinning" />
                              <span>Switching…</span>
                            </>
                          ) : (
                            <>
                              <ArrowRightLeft size={13} />
                              <span>Switch to {p.name}</span>
                            </>
                          )}
                        </button>
                      ) : (
                        <button
                          type="button"
                          className={`fleet-launch-btn ${cardInstalled ? 'installed' : ''} ${cardRunning ? 'running' : ''}`}
                          onClick={(e) => handleCardLaunch(e, p.id)}
                          disabled={isLaunchDisabled}
                          data-testid={`launch-btn-${p.id}`}
                          title={
                            isThisLaunching
                              ? `Launching ${p.name}…`
                              : !cardInstalled
                              ? `${p.name} is not installed on this system`
                              : cardAmbiguous
                              ? `${p.name} detection is ambiguous and requires review`
                              : cardRunning
                              ? `${p.name} is currently running (PID: ${pState?.process?.pid ?? 'Active'})`
                              : `Launch ${p.name}`
                          }
                        >
                          {isThisLaunching ? (
                            <>
                              <RefreshCw size={13} className="spinning" />
                              <span>Launching…</span>
                            </>
                          ) : (
                            <>
                              <Play size={13} fill={cardRunning ? 'currentColor' : 'none'} />
                              <span>
                                {cardRunning
                                  ? `Running (${pState?.process?.pid ? `PID: ${pState.process.pid}` : 'Active'})`
                                  : `Launch ${p.name}`}
                              </span>
                            </>
                          )}
                        </button>
                      )}
                      {cardRunning && (
                        <button
                          type="button"
                          className="fleet-stop-btn"
                          onClick={(e) => handleCardStop(e, p.id)}
                          disabled={isStopping[p.id] || isOldStopping || isBusy}
                          data-testid={`stop-btn-${p.id}`}
                          title={`Stop ${p.name}`}
                        >
                          {isStopping[p.id] || isOldStopping ? (
                            <RefreshCw size={12} className="spinning" />
                          ) : (
                            <Square size={12} />
                          )}
                          <span>Stop</span>
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      </div>

      {/* Switch Confirmation Modal */}
      {pendingSwitchProviderId && (
        <div className="switch-modal-backdrop" onClick={() => setPendingSwitchProviderId(null)}>
          <div className="switch-modal-card" onClick={(e) => e.stopPropagation()}>
            <div className="switch-modal-header">
              <ArrowRightLeft size={20} className="switch-modal-icon" />
              <h3>Confirm Application Switch</h3>
            </div>
            <p className="switch-modal-body">
              Resource Saver Mode is active to protect system memory. Switching to{' '}
              <strong>{PROVIDERS.find((p) => p.id === pendingSwitchProviderId)?.name}</strong> will
              gracefully stop{' '}
              <strong>
                {runningProviders
                  .filter((p) => p.id !== pendingSwitchProviderId)
                  .map((p) => p.name)
                  .join(', ')}
              </strong>
              . Continue?
            </p>
            <div className="switch-modal-actions">
              <button
                type="button"
                className="switch-modal-cancel-btn"
                onClick={() => setPendingSwitchProviderId(null)}
              >
                Cancel
              </button>
              <button
                type="button"
                className="switch-modal-confirm-btn"
                onClick={() => executeSwitch(pendingSwitchProviderId)}
              >
                Switch to {PROVIDERS.find((p) => p.id === pendingSwitchProviderId)?.name}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
