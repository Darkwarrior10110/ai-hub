# AI-Hub: AI Desktop Gateway & Application Lifecycle Manager

AI-Hub is a lightweight native desktop control center for managing installed AI applications (ChatGPT, Gemini, Claude, Grok, Perplexity, Kimi).

Designed specifically for performance and low-memory environments, AI-Hub helps users manage installed AI desktop applications from a single window without keeping multiple heavyweight applications running simultaneously.

---

## Key Features

- **Application Lifecycle Management**: Discover and manage external desktop applications natively.
- **Resource Saver Mode**: Automatically coordinates single-active application execution to protect low-RAM systems.
- **Process Safety**: Enforces graceful SIGTERM shutdown with timeouts. Strictly prohibits indiscriminate `killall` or `pkill` operations.
- **Native Desktop Shell**: Powered by Tauri v2 and Rust for minimal CPU and memory overhead.
- **Privacy & Security**: Operates completely offline with zero API key leakage and zero credential scraping.

---

## Prerequisites

- **Node.js**: v18+ (tested with v20+)
- **Rust Toolchain**: 1.77.2+ (tested with Rust 1.98.0 via `rustup`)
- **System Dependencies** (Debian 13 / Parrot OS 7.3):
  - `libgtk-3-dev`
  - `libwebkit2gtk-4.1-dev`
  - `libssl-dev`
  - `build-essential`
  - `curl`, `wget`, `file`

---

## Build & Run Instructions

### 1. Run Development Server
```bash
# Ensure Rustup toolchain is on PATH if applicable
export PATH="$HOME/.cargo/bin:$PATH"

# Run Tauri dev mode (with Linux DMA-BUF compatibility flags if needed)
WEBKIT_DISABLE_DMABUF_RENDERER=1 WEBKIT_DISABLE_COMPOSITING_MODE=1 npm --prefix apps/desktop run tauri dev
```

### 2. Frontend Verification & Build
```bash
cd apps/desktop
npm test        # Run Vitest test suite
npm run lint    # Run ESLint checks
npm run build   # Compile TypeScript and Vite production bundle
```

### 3. Rust Backend Verification & Tests
```bash
cd apps/desktop/src-tauri
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --check   # Check Rust code formatting
cargo check         # Verify Rust crate compilation
cargo test          # Execute Rust unit test suite
```

---

## Architecture

For in-depth architecture documentation, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
