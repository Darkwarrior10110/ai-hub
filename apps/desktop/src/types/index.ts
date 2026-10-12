export type Theme = 'light' | 'dark' | 'system';
export type ProviderId = 'chatgpt' | 'gemini' | 'claude' | 'grok' | 'perplexity' | 'kimi';

export interface Provider {
  id: ProviderId;
  name: string;
  color: string;
  description: string;
}

export const PROVIDERS: Provider[] = [
  { id: 'chatgpt', name: 'ChatGPT', color: '#10a37f', description: 'OpenAI' },
  { id: 'gemini', name: 'Gemini', color: '#4285f4', description: 'Google' },
  { id: 'claude', name: 'Claude', color: '#d97706', description: 'Anthropic' },
  { id: 'grok', name: 'Grok', color: '#1d9bf0', description: 'xAI' },
  { id: 'perplexity', name: 'Perplexity', color: '#20b2aa', description: 'Perplexity AI' },
  { id: 'kimi', name: 'Kimi', color: '#7c3aed', description: 'Moonshot AI' },
];

export const getProviderInitial = (id: ProviderId): string => {
  const map: Record<ProviderId, string> = {
    chatgpt: 'G',
    gemini: 'G',
    claude: 'C',
    grok: 'X',
    perplexity: 'P',
    kimi: 'K',
  };
  return map[id];
};
