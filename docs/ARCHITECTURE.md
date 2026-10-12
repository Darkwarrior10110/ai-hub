# AI-Hub System Architecture

## 1. Product Definition & Vision

**AI-Hub** is a native, lightweight **AI Desktop Gateway, Application Lifecycle Manager, and Resource Monitor**.

Rather than recreating proprietary web chat interfaces or requiring heavy browser tabs, AI-Hub provides a centralized native control center for managing installed AI desktop applications (ChatGPT, Gemini, Claude, Grok, Perplexity, Kimi).

Its core purpose is to optimize desktop workflow and resource utilization—especially on resource-constrained computers (e.g., 4 GB RAM systems)—by ensuring that multiple heavyweight AI applications are not unnecessarily executing simultaneously.

---

## 2. Component Boundaries & High-Level Architecture

```
React Desktop UI (Vite / TypeScript)
        │
        ▼ Typed Tauri Commands & Events (IPC)
Rust Application Controller
        │
        ├── Provider Registry (Catalog of 6 AI Providers)
        ├── Application Discovery Engine (Snap, Desktop Entries, Executables)
        ├── Runtime Manager
        │     ├── Linux Runtime (Snap, Procfs, SIGTERM)
        │     └── Windows Runtime (Platform boundary stub)
        ├── Process Lifecycle Manager (Safe tracking, Graceful shutdown)
        ├── Resource Monitor (Metrics sampling, low overhead)
        └── Runtime Configuration (Resource Saver Mode, Timeouts)
        │
        ▼
Operating System (Linux Debian/Parrot, Windows)
```

### Separation of Responsibilities

1. **React Desktop UI (`apps/desktop/src`)**:
   - Displays real-time application lifecycle state received from the Rust backend.
   - Dispatches user intents (launch, stop, switch, settings changes) via strongly typed Tauri IPC.
   - **Rule**: The UI never fabricates application status, never executes shell commands, and never handles raw PIDs.

2. **Rust Backend (`apps/desktop/src-tauri/src`)**:
   - Single source of truth for all application detection, process monitoring, and lifecycle states.
   - Safely interfaces with OS primitives (filesystem, `/proc`, process signals).
   - Validates all arguments and returns structured error types (`RuntimeError`).

3. **Legacy Provider SDK (`packages/provider-sdk`)**:
   - Isolated from the primary application gateway. Retained as optional protocol infrastructure without being a dependency of the core desktop gateway lifecycle engine.

---

## 3. Rust Domain Models & Lifecycle State Machine

### Lifecycle States
An application exists in one of the following explicit states:
- `not_installed`: Application is not present on the host system.
- `installed`: Executable or package detected on the system and ready for launch.
- `ready`: Application verified and ready to be launched.
- `starting`: Process launch initiated; waiting for confirmation.
- `running`: Process verified alive and actively tracked.
- `stopping`: Graceful termination requested; waiting within timeout.
- `switching`: Coordinated transition from current managed app to target app.
- `error`: Transition or execution error occurred.
- `unknown`: State pending initial discovery.

### Process Ownership Boundaries
To prevent terminating unrelated software, AI-Hub distinguishes:
- `StartedByAIHub`: Explicitly spawned by AI-Hub; candidate for managed lifecycle.
- `AlreadyRunningExternally`: Discovered already running before AI-Hub intervention.
- `DiscoveredRunning`: Running process detected by ambient scans.
- `OwnershipUnknown`: Disclaimed process; destructive termination strictly disallowed.

---

## 4. Process Safety & Resource Saver Mode

### Process Safety Rules
- **No Blanket Process Killing**: Commands like `killall`, `pkill`, or `kill -9` based solely on process names are strictly forbidden.
- **Graceful Termination First**: Managed processes are sent a graceful termination request first (e.g., standard SIGTERM on Linux).
- **Graceful Timeout**: A configurable wait period (default: 5 seconds) allows applications to persist user state and exit cleanly.
- **Force Termination Restricted**: Force termination (`SIGKILL`) is **disabled by default** and only permissible for verified `StartedByAIHub` processes when explicitly enabled by user policy.

### Resource Saver Mode
- **Goal**: Maintain only **one** active managed AI desktop application at a time.
- **Switching Workflow**:
  1. Validate target application availability.
  2. Request graceful shutdown of currently active provider.
  3. Await termination verification within configured timeout.
  4. Launch target provider upon successful cleanup.
  5. If the current application cannot stop safely, abort the switch to preserve user data.

---

## 5. Platform Runtime Implementations

### Linux Runtime (`apps/desktop/src-tauri/src/runtime/linux.rs` & `snap.rs`)
- **Discovery Strategy**:
  1. **Snap Engine (`snap.rs`)**:
     - Locates `/usr/bin/snap` or scans `$PATH` deterministically without shell invocation.
     - Runs bounded query (`snap list --color=never --unicode=never`) with strict 5-second timeout and process cleanup on timeout.
     - Pure parser handles header detection, variable spacing, notes (`-` vs text), and publisher verification flags (`canonical**`).
     - Rejects false matches with a generic-naming guard (words like `ai`, `chat`, `assistant`, `desktop` alone never match).
     - Reports 5 discrete states: `Confirmed`, `Ambiguous`, `NotInstalled`, `Unavailable`, `Failed`.
     - Strictly decouples installation discovery from runtime process state (discovered packages are never set to `Running`).
  2. **Desktop Entries**: Will inspect system and user `.desktop` files in Milestone 6.
  3. **Executables**: Direct validated filesystem checks in `/snap/bin/` fallback.
- **Process Verification**: Non-intrusive existence checks via `/proc/<PID>`.

### Windows Runtime (`apps/desktop/src-tauri/src/runtime/windows.rs`)
- Isolated behind the `RuntimePlatform` trait.
- Returns structured `PlatformNotSupported` errors when compiled on non-Windows platforms.
- Windows-specific implementation will be introduced and tested on Windows hosts in Milestone 8.

---

## 6. Security & Capabilities Boundary

- **Tauri IPC Whitelist**: Only explicit, strongly typed commands are exposed (`list_providers`, `get_provider_state`, `get_runtime_settings`, `update_runtime_settings`, etc.).
- **No Arbitrary Shell Execution**: The frontend cannot request arbitrary executable paths or arbitrary process termination.
- **Input Validation**: All provider IDs and settings are parsed into Rust enums and bounded ranges before processing.
