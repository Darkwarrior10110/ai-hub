import { ProviderManager } from '../manager/ProviderManager';
import { ChatGPTAdapter } from '../adapters/chatgpt';
import { GeminiAdapter } from '../adapters/gemini';
import { ClaudeAdapter } from '../adapters/claude';
import { GrokAdapter } from '../adapters/grok';
import { PerplexityAdapter } from '../adapters/perplexity';
import { KimiAdapter } from '../adapters/kimi';

export const providerManager = new ProviderManager();

providerManager.registerProvider(new ChatGPTAdapter());
providerManager.registerProvider(new GeminiAdapter());
providerManager.registerProvider(new ClaudeAdapter());
providerManager.registerProvider(new GrokAdapter());
providerManager.registerProvider(new PerplexityAdapter());
providerManager.registerProvider(new KimiAdapter());
