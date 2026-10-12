import { useState } from 'react';
import { useUI } from '../../providers/UIProvider';
import { PROVIDERS, getProviderInitial } from '../../types';
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
  Info
} from 'lucide-react';
import './GatewayDashboard.css';

export function GatewayDashboard() {
  const {
    selectedProvider,
    setSelectedProvider,
    managedApps,
    runtimeSettings,
    launchProvider,
    stopProvider,
    refreshProviderStates,
  } = useUI();

  const providerMeta = PROVIDERS.find(p => p.id === selectedProvider) || PROVIDERS[0];
  const appState = managedApps[selectedProvider];
  const [actionError, setActionError] = useState<string | null>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const handleLaunch = async () => {
    setActionError(null);
    try {
      await launchProvider(selectedProvider);
    } catch (err: unknown) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleStop = async () => {
    setActionError(null);
    try {
      await stopProvider(selectedProvider);
    } catch (err: unknown) {
      setActionError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRefresh = async () => {
    setIsRefreshing(true);
    setActionError(null);
    try {
      await refreshProviderStates();
    } finally {
      setIsRefreshing(false);
    }
  };

  const isInstalled = appState?.lifecycle_state === 'installed' || appState?.lifecycle_state === 'ready';
  const isRunning = appState?.lifecycle_state === 'running';

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
          </div>
          <div className="header-status-row">
            <span className={`status-pill ${appState?.lifecycle_state || 'unknown'}`}>
              <span className="status-indicator-dot" />
              {appState?.lifecycle_state === 'installed' && 'Installed (Ready)'}
              {appState?.lifecycle_state === 'not_installed' && 'Not Installed'}
              {appState?.lifecycle_state === 'running' && 'Running'}
              {appState?.lifecycle_state === 'starting' && 'Starting…'}
              {appState?.lifecycle_state === 'stopping' && 'Stopping…'}
              {appState?.lifecycle_state === 'switching' && 'Switching…'}
              {appState?.lifecycle_state === 'unknown' && 'Detecting…'}
              {appState?.lifecycle_state === 'error' && 'Error'}
            </span>
            {runtimeSettings.resource_saver_mode && (
              <span className="resource-saver-pill" title="Keeps only one AI application running to protect low-RAM systems">
                <ShieldCheck size={13} />
                <span>Resource Saver Active</span>
              </span>
            )}
          </div>
        </div>

        <div className="header-actions">
          <button
            className="refresh-btn"
            onClick={handleRefresh}
            disabled={isRefreshing}
            title="Scan installed applications"
          >
            <RefreshCw size={15} className={isRefreshing ? 'spinning' : ''} />
            <span>Scan System</span>
          </button>
        </div>
      </header>

      {/* Main Content Area */}
      <div className="gateway-content">
        {/* Error notification banner if any action failed */}
        {actionError && (
          <div className="gateway-alert-banner">
            <AlertCircle size={18} />
            <div className="alert-content">
              <strong>Application Lifecycle Notice</strong>
              <p>{actionError}</p>
            </div>
            <button className="alert-close" onClick={() => setActionError(null)}>×</button>
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
                  <button
                    className="launch-btn primary"
                    onClick={handleLaunch}
                    style={{ backgroundColor: providerMeta.color }}
                  >
                    <Play size={16} fill="currentColor" />
                    <span>Launch {providerMeta.name}</span>
                  </button>
                  <button
                    className="stop-btn secondary"
                    onClick={handleStop}
                    disabled={!isRunning}
                  >
                    <Square size={15} />
                    <span>Stop Process</span>
                  </button>
                </>
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

        {/* System Monitoring & Resource Guard Grid */}
        <section className="metrics-grid">
          <div className="metric-card">
            <div className="metric-icon">
              <Cpu size={18} />
            </div>
            <div className="metric-details">
              <span className="metric-label">Resource Saver Mode</span>
              <span className="metric-value">
                {runtimeSettings.resource_saver_mode ? 'Enabled (Single App)' : 'Disabled'}
              </span>
              <span className="metric-hint">Prevents running multiple heavyweight AI apps simultaneously</span>
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">
              <ShieldCheck size={18} />
            </div>
            <div className="metric-details">
              <span className="metric-label">Process Safety Policy</span>
              <span className="metric-value">Graceful ({runtimeSettings.graceful_shutdown_timeout_secs}s timeout)</span>
              <span className="metric-hint">Zero dangerous killall commands. Force-kill disabled by default</span>
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">
              <Layers size={18} />
            </div>
            <div className="metric-details">
              <span className="metric-label">Tracked Processes</span>
              <span className="metric-value">
                {appState?.process ? `PID: ${appState.process.pid}` : 'None active'}
              </span>
              <span className="metric-hint">Real-time OS process monitoring</span>
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
              const state = managedApps[p.id];
              const installed = state?.lifecycle_state === 'installed' || state?.lifecycle_state === 'ready';
              const isSelected = selectedProvider === p.id;

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
                      <span className="fleet-vendor">{p.description}</span>
                    </div>
                    <div className="fleet-badge-wrap">
                      {installed ? (
                        <span className="fleet-installed-tag" title="Installed on system">
                          <CheckCircle2 size={13} />
                          <span>Detected</span>
                        </span>
                      ) : (
                        <span className="fleet-missing-tag" title="Not found on system">
                          <XCircle size={13} />
                          <span>Not Installed</span>
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      </div>
    </div>
  );
}
