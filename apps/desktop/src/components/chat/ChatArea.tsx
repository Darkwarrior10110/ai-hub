import { useUI } from '../../providers/UIProvider';
import { PROVIDERS } from '../../types';
import { MessageComposer } from '../composer/MessageComposer';
import { Sparkles, ArrowRight, Plug, AlertCircle } from 'lucide-react';
import './ChatArea.css';
import { useState } from 'react';

export function ChatArea() {
  const { selectedProvider, launchProvider, managedApps } = useUI();
  const provider = PROVIDERS.find(p => p.id === selectedProvider)!;
  const appState = managedApps[selectedProvider];
  const isRunning = appState?.lifecycle_state === 'running';
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const handleConnect = async () => {
    setErrorMsg(null);
    try {
      await launchProvider(selectedProvider);
    } catch (e: unknown) {
      setErrorMsg(e instanceof Error ? e.message : 'Failed to launch provider.');
    }
  };

  return (
    <div className="chat-area">
      {/* Header */}
      <header className="chat-header">
        <div className="chat-header-left">
          <div
            className="header-provider-badge"
            style={{ backgroundColor: provider.color }}
          >
            <Sparkles size={14} />
          </div>
          <div className="header-provider-info">
            <span className="header-provider-name">{provider.name}</span>
            <span className="header-provider-status">
              <span className={`status-dot ${isRunning ? 'online' : 'offline'}`} />
              {isRunning ? 'Running' : 'Ready'}
            </span>
          </div>
        </div>
        <div className="chat-header-right">
           <button 
             className="connect-btn" 
             onClick={handleConnect}
             disabled={isRunning}
           >
             <Plug size={14} />
             <span>{isRunning ? 'Running' : 'Launch'}</span>
           </button>
        </div>
      </header>

      {/* Messages / Empty State */}
      <div className="chat-messages">
        <div className="empty-state">
          <div className="empty-glow" style={{ background: `radial-gradient(circle, ${provider.color}15 0%, transparent 70%)` }} />
          <div className="empty-icon" style={{ borderColor: `${provider.color}30` }}>
            <Sparkles size={32} color={provider.color} />
          </div>
          <h2 className="empty-title">Start a conversation</h2>
          <p className="empty-subtitle">
            Ask {provider.name} anything — coding, research, ideas, debugging, or architecture.
          </p>

          {errorMsg && (
            <div className="error-card">
              <AlertCircle size={16} />
              <span>{errorMsg}</span>
            </div>
          )}

          <div className="suggestion-chips">
            {[
              'Explain a concept',
              'Debug my code',
              'Compare approaches',
              'Research a topic',
            ].map(suggestion => (
              <button key={suggestion} className="suggestion-chip">
                <span>{suggestion}</span>
                <ArrowRight size={14} />
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Composer */}
      <div className="chat-composer-wrapper">
        <MessageComposer />
      </div>
    </div>
  );
}
