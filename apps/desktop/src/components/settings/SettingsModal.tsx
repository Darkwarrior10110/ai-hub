import { useState } from 'react';
import { X, Moon, Sun, Monitor, ChevronRight } from 'lucide-react';
import { useUI } from '../../providers/UIProvider';
import type { Theme } from '../../types';
import './SettingsModal.css';

const TABS = [
  { id: 'general', label: 'General' },
  { id: 'appearance', label: 'Appearance' },
  { id: 'providers', label: 'Providers' },
  { id: 'account', label: 'Account' },
  { id: 'privacy', label: 'Privacy' },
  { id: 'data-sync', label: 'Data & Sync' },
] as const;

type TabId = typeof TABS[number]['id'];

const themeOptions: { id: Theme; label: string; icon: typeof Sun }[] = [
  { id: 'light', label: 'Light', icon: Sun },
  { id: 'dark', label: 'Dark', icon: Moon },
  { id: 'system', label: 'System', icon: Monitor },
];

export function SettingsModal() {
  const { isSettingsOpen, setSettingsOpen, theme, setTheme } = useUI();
  const [activeTab, setActiveTab] = useState<TabId>('general');

  if (!isSettingsOpen) return null;

  return (
    <div className="settings-overlay" onClick={() => setSettingsOpen(false)}>
      <div className="settings-dialog" onClick={e => e.stopPropagation()}>
        {/* Sidebar */}
        <div className="settings-nav">
          <h2 className="settings-title">Settings</h2>
          <nav className="settings-tabs">
            {TABS.map(tab => (
              <button
                key={tab.id}
                className={`settings-tab ${activeTab === tab.id ? 'active' : ''}`}
                onClick={() => setActiveTab(tab.id)}
              >
                <span>{tab.label}</span>
                <ChevronRight size={14} className="tab-arrow" />
              </button>
            ))}
          </nav>
          <div className="settings-nav-footer">
            <span className="version-label">AI-Hub v0.1.0-alpha</span>
          </div>
        </div>

        {/* Content */}
        <div className="settings-content">
          <div className="settings-content-header">
            <h3>{TABS.find(t => t.id === activeTab)?.label}</h3>
            <button className="settings-close" onClick={() => setSettingsOpen(false)}>
              <X size={18} />
            </button>
          </div>

          <div className="settings-scroll" key={activeTab}>
            {activeTab === 'appearance' && (
              <section className="settings-section">
                <h4>Theme</h4>
                <p className="section-desc">Choose your preferred appearance.</p>
                <div className="theme-grid">
                  {themeOptions.map(opt => {
                    const Icon = opt.icon;
                    return (
                      <button
                        key={opt.id}
                        className={`theme-card ${theme === opt.id ? 'selected' : ''}`}
                        onClick={() => setTheme(opt.id)}
                      >
                        <div className="theme-card-icon">
                          <Icon size={22} />
                        </div>
                        <span className="theme-card-label">{opt.label}</span>
                        {theme === opt.id && <div className="theme-check" />}
                      </button>
                    );
                  })}
                </div>
              </section>
            )}

            {activeTab === 'account' && (
              <section className="settings-section">
                <h4>Authentication</h4>
                <p className="section-desc">Sign in to sync your conversations and settings across devices.</p>
                <div className="coming-soon-card">
                  <div className="coming-soon-badge">Coming Soon</div>
                  <p>Authentication will be implemented in a later phase.</p>
                  <p className="coming-soon-sub">Google sign-in and email authentication are planned.</p>
                </div>
              </section>
            )}

            {activeTab === 'providers' && (
              <section className="settings-section">
                <h4>Provider Configuration</h4>
                <p className="section-desc">Manage your API keys and provider settings.</p>
                <div className="coming-soon-card">
                  <div className="coming-soon-badge">Coming Soon</div>
                  <p>Provider integrations will be added incrementally.</p>
                  <p className="coming-soon-sub">Each provider will have its own secure configuration.</p>
                </div>
              </section>
            )}

            {activeTab === 'general' && (
              <section className="settings-section">
                <h4>Application</h4>
                <p className="section-desc">General application settings and preferences.</p>
                <div className="coming-soon-card">
                  <div className="coming-soon-badge">Coming Soon</div>
                  <p>Language, startup behavior, and notification preferences will be available here.</p>
                </div>
              </section>
            )}

            {activeTab === 'privacy' && (
              <section className="settings-section">
                <h4>Privacy & Security</h4>
                <p className="section-desc">Control how your data is handled.</p>
                <div className="coming-soon-card">
                  <div className="coming-soon-badge">Coming Soon</div>
                  <p>Data retention, conversation privacy, and security settings will be available here.</p>
                </div>
              </section>
            )}

            {activeTab === 'data-sync' && (
              <section className="settings-section">
                <h4>Data & Synchronization</h4>
                <p className="section-desc">Manage your conversation history and cross-device sync.</p>
                <div className="coming-soon-card">
                  <div className="coming-soon-badge">Coming Soon</div>
                  <p>Cloud synchronization, data export, and storage management will be available here.</p>
                </div>
              </section>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
