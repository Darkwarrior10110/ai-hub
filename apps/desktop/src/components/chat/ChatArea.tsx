import { useUI } from '../../providers/UIProvider';
import { PROVIDERS } from '../../types';
import { providerManager } from '../../providers/registry';
import { ConnectionState, AuthenticationNotSupported } from '@ai-hub/provider-sdk';
import { MessageComposer } from '../composer/MessageComposer';
import { Sparkles, ArrowRight, Plug, AlertCircle } from 'lucide-react';
import './ChatArea.css';
import { useState } from 'react';

export function ChatArea() {
  const { selectedProvider, connectProvider } = useUI();
  const provider = PROVIDERS.find(p => p.id === selectedProvider)!;
  const adapter = providerManager.getProvider(selectedProvider);
  const connectionState = adapter.getConnectionState();
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const handleConnect = async () => {
    setErrorMsg(null);
    try {
      await connectProvider(selectedProvider);
    } catch (e) {
      if (e instanceof AuthenticationNotSupported) {
        setErrorMsg(e.message);
      } else {
        setErrorMsg('Failed to connect.');
      }
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
              <span className={`status-dot ${connectionState === ConnectionState.CONNECTED ? 'online' : connectionState === ConnectionState.ERROR ? 'error' : 'offline'}`} />
              {connectionState === ConnectionState.CONNECTED ? 'Connected' : 
               connectionState === ConnectionState.CONNECTING ? 'Connecting...' : 
               connectionState === ConnectionState.ERROR ? 'Error' : 'Not connected'}
            </span>
          </div>
        </div>
        <div className="chat-header-right">
           <button 
             className="connect-btn" 
             onClick={handleConnect}
             disabled={connectionState === ConnectionState.CONNECTING || connectionState === ConnectionState.CONNECTED}
           >
             <Plug size={14} />
             <span>{connectionState === ConnectionState.CONNECTED ? 'Connected' : 'Connect'}</span>
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
