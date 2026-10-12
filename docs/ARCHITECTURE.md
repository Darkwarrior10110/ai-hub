# AI-Hub Provider Architecture

## Overview
AI-Hub is built with a highly decoupled provider architecture. The Desktop UI relies strictly on an abstraction layer (the `ProviderManager`) and never interacts with provider-specific APIs directly.

### Core Structure
1. **Provider SDK (`packages/provider-sdk`)**: 
   Contains all interfaces, types, and error classes.
2. **Provider Manager (`apps/desktop/src/providers/manager`)**: 
   Central registry and state manager for available AI providers.
3. **Provider Adapters (`apps/desktop/src/providers/adapters`)**: 
   Classes implementing the `ProviderAdapter` interface for specific services (ChatGPT, Gemini, etc.).

---

## Architectural Boundaries

### 1. Provider Adapter Interface
Every provider must implement `ProviderAdapter`, which enforces:
- `info`: Metadata (ID, Name, Version).
- `capabilities`: Explicit flags for features (e.g., `streaming`, `webSearch`).
- `supportedAuthStrategies`: A list of authentication strategies supported by this provider.
- **Connection Lifecycle**: `connect()`, `disconnect()`, `restoreConnection()`, `getConnectionState()`.
- **Messaging**: `sendMessage()`

### 2. Capabilities Model
Do not assume providers support everything. Instead, use explicit capability flags defined in `ProviderCapabilities`:
- `chat`
- `streaming`
- `attachments`
- `imageInput`
- `webSearch`
- `history`
- `modelSelection`

### 3. Authentication Abstraction
Providers use entirely different authentication mechanisms. The SDK provides an `AuthenticationStrategy` enum representing methods like `OAUTH`, `API_KEY`, or `DEVICE_AUTHORIZATION`. The actual implementation of these strategies is deferred to a future secure credential module.

### 4. Secure Credential Architecture (Future)
Credentials will be stored securely using the OS keychain via Tauri/Rust. 
The abstraction `SecureCredentialStore` exists in the SDK to define how adapters will retrieve tokens:
- `saveCredential(providerId, data)`
- `getCredential(providerId)`
- `deleteCredential(providerId)`
*(Important: Credentials are never stored in localStorage, React state, or plaintext files).*

### 5. Provider Isolation
Providers are instantiated as separate class instances. A failure, crash, or rate-limit in `ChatGPTAdapter` will only trigger a state change for that specific adapter. The `ProviderManager` handles switching context and ensures isolation.

---

## Adding a New Provider
To add a future provider:
1. Create a new file in `providers/adapters/`.
2. Implement the `ProviderAdapter` interface.
3. Register it in `providers/registry/index.ts`.
4. Ensure no unauthorized or unofficial authentication mechanisms are used (only use provider-approved connection methods).
