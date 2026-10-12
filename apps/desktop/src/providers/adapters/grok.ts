import { ProviderAdapter, ProviderInfo, ProviderCapabilities, AuthenticationStrategy, ConnectionState, Message, AuthenticationNotSupported } from '@ai-hub/provider-sdk';

export class GrokAdapter implements ProviderAdapter {
  info: ProviderInfo = { id: 'grok', name: 'Grok', version: '1.0.0' };
  
  capabilities: ProviderCapabilities = {
    chat: true,
    streaming: true,
    attachments: false,
    imageInput: false,
    webSearch: false,
    history: true,
    modelSelection: false
  };
  
  supportedAuthStrategies: AuthenticationStrategy[] = [
    AuthenticationStrategy.OAUTH, 
    AuthenticationStrategy.API_KEY
  ];
  
  private state: ConnectionState = ConnectionState.DISCONNECTED;

  getConnectionState(): ConnectionState {
    return this.state;
  }

  async connect(): Promise<void> {
    this.state = ConnectionState.CONNECTING;
    // Architecture phase only: do NOT implement real auth yet
    await new Promise(resolve => setTimeout(resolve, 300));
    this.state = ConnectionState.ERROR;
    throw new AuthenticationNotSupported('Connection mechanism not configured yet for Grok.');
  }

  async disconnect(): Promise<void> {
    this.state = ConnectionState.DISCONNECTED;
  }

  async restoreConnection(): Promise<void> {
    this.state = ConnectionState.DISCONNECTED;
  }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async sendMessage(_message: Message): Promise<Message> {
    throw new Error('Not connected');
  }
}
