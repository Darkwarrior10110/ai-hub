import { Plus, Settings, User, PanelLeftClose, PanelLeftOpen, Zap } from 'lucide-react';
import { useUI } from '../../providers/UIProvider';
import { PROVIDERS, getProviderInitial } from '../../types';
import './Sidebar.css';

export function Sidebar() {
  const {
    selectedProvider, setSelectedProvider,
    setSettingsOpen,
    isSidebarCollapsed, toggleSidebar,
  } = useUI();

  return (
    <aside className={`sidebar ${isSidebarCollapsed ? 'collapsed' : ''}`}>
      {/* Header */}
      <div className="sidebar-header">
        <div className="brand">
          <div className="brand-icon">
            <Zap size={18} />
          </div>
          {!isSidebarCollapsed && <span className="brand-name">AI-Hub</span>}
        </div>
        <button
          className="collapse-btn"
          onClick={toggleSidebar}
          title={isSidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {isSidebarCollapsed ? <PanelLeftOpen size={18} /> : <PanelLeftClose size={18} />}
        </button>
      </div>

      {/* New Chat */}
      <div className="sidebar-actions">
        <button className="new-chat-btn" title="New Chat">
          <Plus size={18} />
          {!isSidebarCollapsed && <span>New Chat</span>}
        </button>
      </div>

      {/* Providers */}
      <div className="sidebar-section">
        {!isSidebarCollapsed && <h3 className="section-label">Providers</h3>}
        <ul className="provider-list">
          {PROVIDERS.map(p => (
            <li
              key={p.id}
              className={`provider-item ${selectedProvider === p.id ? 'active' : ''}`}
              onClick={() => setSelectedProvider(p.id)}
              title={isSidebarCollapsed ? p.name : undefined}
            >
              <div
                className="provider-badge"
                style={{
                  backgroundColor: selectedProvider === p.id ? p.color : 'transparent',
                  borderColor: p.color,
                  color: selectedProvider === p.id ? '#fff' : p.color,
                }}
              >
                {getProviderInitial(p.id)}
              </div>
              {!isSidebarCollapsed && (
                <div className="provider-info">
                  <span className="provider-name">{p.name}</span>
                  <span className="provider-desc">{p.description}</span>
                </div>
              )}
              {!isSidebarCollapsed && selectedProvider === p.id && (
                <div className="active-dot" style={{ backgroundColor: p.color }} />
              )}
            </li>
          ))}
        </ul>
      </div>

      <div className="sidebar-spacer" />

      {/* Footer */}
      <div className="sidebar-footer">
        <button
          className="footer-btn"
          onClick={() => setSettingsOpen(true)}
          title="Settings"
        >
          <Settings size={18} />
          {!isSidebarCollapsed && <span>Settings</span>}
        </button>
        <button className="footer-btn" title="Account">
          <User size={18} />
          {!isSidebarCollapsed && <span>Account</span>}
        </button>
      </div>
    </aside>
  );
}
