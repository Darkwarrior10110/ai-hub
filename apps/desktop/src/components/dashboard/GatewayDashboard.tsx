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
    discoverySnapshot,
    isScanning,
    scanApplications,
    launchProvider,
    stopProvider,
  } = useUI();

  const providerMeta = PROVIDERS.find(p => p.id === selectedProvider) || PROVIDERS[0];
  const appState = managedApps[selectedProvider];
  const discoveryResult = discoverySnapshot?.results.find(r => r.provider_id === selectedProvider);
  const [actionError, setActionError] = useState<string | null>(null);

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

  const handleScan = async () => {
    setActionError(null);
    await scanApplications();
  };

  const isConfirmed = discoveryResult?.status === 'confirmed';
  const isInstalled = appState?.lifecycle_state === 'installed' || isConfirmed;
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
            onClick={handleScan}
            disabled={isScanning}
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
                  </div>
                  <p className="milestone-notice">
                    ✓ Discovery Verified: Application detected on host. Native process launch coordination will be connected in Milestone 3.
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
              const disc = discoverySnapshot?.results.find((r) => r.provider_id === p.id);
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
                </div>
              );
            })}
          </div>
        </section>
      </div>
    </div>
  );
}
