# ⚡ AI-Hub — AI Desktop Gateway & Application Lifecycle Manager

<p align="center">
  <strong>One control center. Multiple AI applications. Less wasted RAM.</strong>
</p>

<p align="center">
  A lightweight native desktop application for discovering, launching, monitoring, and managing installed AI applications — built for systems where every megabyte matters.
</p>

<p align="center">
  <a href="https://github.com/Darkwarrior10110/ai-hub"><strong>GitHub Repository</strong></a>
  ·
  <a href="https://github.com/Darkwarrior10110/ai-hub/issues">Report a Bug</a>
  ·
  <a href="https://github.com/Darkwarrior10110/ai-hub/issues">Request a Feature</a>
</p>

---

## 📌 Table of Contents

- [🌌 Overview](#-overview)
- [✨ Key Features](#-key-features)
- [🎯 Why AI-Hub?](#-why-ai-hub)
- [🏗️ Architecture](#️-architecture)
- [🧰 Technology Stack](#-technology-stack)
- [💻 System Requirements](#-system-requirements)
- [🚀 Getting Started](#-getting-started)
- [🧪 Verification and Testing](#-verification-and-testing)
- [🔐 Security and Privacy](#-security-and-privacy)
- [🗺️ Roadmap](#️-roadmap)
- [🤝 Contributing](#-contributing)
- [📄 License](#-license)

---

## 🌌 Overview

AI-Hub is a **native desktop control center for installed AI applications** such as ChatGPT, Gemini, Claude, Grok, Perplexity, and Kimi.

Running multiple heavyweight applications simultaneously can consume substantial system memory and resources. AI-Hub aims to make managing these applications easier by bringing their lifecycle controls into a single desktop interface.

Instead of keeping every application running, AI-Hub is designed around coordinated application lifecycle management and resource-conscious execution.

> [!IMPORTANT]
> **AI-Hub is an application lifecycle manager, not an AI chatbot aggregator.** It manages external applications rather than replacing them with another interface for sending prompts to multiple AI providers.

### 💡 The core idea

- **Discover** supported applications installed on your system.
- **Manage** their launch and shutdown through a centralized interface.
- **Conserve resources** by coordinating application execution.
- **Stay in control** with explicit process-management behavior instead of indiscriminate process termination.

---

## ✨ Key Features

| Feature | Description |
|---|---|
| 🖥️ **Application Lifecycle Management** | Discover and manage supported external AI desktop applications. |
| 🧠 **Resource Saver Mode** | Coordinate application execution to help reduce unnecessary concurrent resource usage. |
| 🛡️ **Process Safety** | Use graceful `SIGTERM` shutdown with timeout handling rather than indiscriminate `killall` or `pkill` operations. |
| ⚡ **Native Desktop Shell** | Built with Tauri v2 and Rust to keep the application lightweight. |
| 🔒 **Privacy-Focused Design** | Avoids API-key collection and credential scraping as part of its application-management approach. |
| 🧩 **Modular Architecture** | Separates desktop commands, application discovery, lifecycle management, and platform-specific runtime behavior. |

> **Design principle:** AI-Hub should manage applications deliberately, preserve user control, and avoid unnecessary background work.

---

## 🎯 Why AI-Hub?

Many developers and students use several AI applications throughout their day. On a resource-constrained computer, keeping all of them open simultaneously can make the entire system less responsive.

AI-Hub focuses on the operational problem: **managing the applications you already use without unnecessarily keeping them all running.**

| Without centralized management | With AI-Hub |
|---|---|
| Switching between separate application windows | A centralized management interface |
| Manually handling application launch and shutdown | Coordinated lifecycle controls |
| Unnecessary applications may remain running | Resource-conscious execution is the goal |
| Different management behavior across components | A shared lifecycle-management architecture |

*Actual resource savings depend on the applications installed, their runtime behavior, and the lifecycle policies supported by the current implementation.*

---

## 🏗️ Architecture

AI-Hub uses a native desktop architecture with a TypeScript frontend and a Rust backend.

```text
┌──────────────────────────────────────────┐
│              Desktop UI                  │
│             React + TypeScript           │
└────────────────────┬─────────────────────┘
                     │
                     │ Tauri IPC
                     ▼
┌──────────────────────────────────────────┐
│              Rust Backend                │
│                                          │
│  ┌────────────┐   ┌───────────────────┐  │
│  │  Commands  │──▶│ Lifecycle Manager │  │
│  └────────────┘   └─────────┬─────────┘  │
│                             │            │
│              ┌──────────────┴─────────┐  │
│              ▼                        ▼  │
│        Application Registry     Runtime  │
│                                 Adapters │
└─────────────────────────────┬────────────┘
                              │
                              ▼
┌──────────────────────────────────────────┐
│       Installed External Applications    │
└──────────────────────────────────────────┘
```

*Conceptual overview; consult the implementation and architecture documentation for exact module responsibilities.*

### 📂 Main project structure

```text
ai-hub/
├── apps/
│   └── desktop/
│       ├── src/                 # React + TypeScript interface
│       └── src-tauri/
│           └── src/
│               ├── commands/   # Desktop IPC commands
│               ├── domain/     # Core domain models
│               ├── manager/    # Application lifecycle logic
│               ├── registry/   # Application discovery and registry
│               └── runtime/    # Platform-specific runtime behavior
├── docs/
│   └── ARCHITECTURE.md         # Detailed architecture documentation
└── README.md
```

### 🔄 Application lifecycle

The intended lifecycle workflow is:

1. **Discover** — identify supported installed applications.
2. **Validate** — check the requested operation and application state.
3. **Execute** — perform the relevant lifecycle operation.
4. **Monitor** — track process state where supported.
5. **Release resources** — request graceful shutdown when appropriate.

The exact behavior depends on the platform runtime and the capabilities implemented for each application.

📖 **Learn more:** [Architecture Documentation](https://github.com/Darkwarrior10110/ai-hub/blob/main/docs/ARCHITECTURE.md)

---

## 🧰 Technology Stack

| Technology | Role |
|---|---|
| [Tauri v2](https://v2.tauri.app/) | Native desktop application framework |
| [Rust](https://www.rust-lang.org/) | Backend logic and process lifecycle management |
| [React](https://react.dev/) | Desktop user interface |
| [TypeScript](https://www.typescriptlang.org/) | Typed frontend development |
| [Vite](https://vite.dev/) | Frontend development and production bundling |
| [Vitest](https://vitest.dev/) | Frontend testing |
| [Cargo](https://doc.rust-lang.org/cargo/) | Rust builds and tests |

---

## 💻 System Requirements

### Software prerequisites

| Dependency | Requirement |
|---|---|
| Node.js | v18+; tested with v20+ |
| Rust toolchain | 1.77.2+; project previously tested with Rust 1.98.0 |
| Cargo | Included with the Rust toolchain |
| Git | Required to clone the repository |

### Linux system dependencies

The following packages are required for the documented Debian 13 / Parrot OS 7.3 development environment:

```bash
sudo apt update

sudo apt install -y \
  build-essential \
  curl \
  wget \
  file \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libssl-dev
```

> [!NOTE]
> Build dependencies can vary by distribution and installed Tauri components. Consult the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system.

---

## 🚀 Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/Darkwarrior10110/ai-hub.git

cd ai-hub
```

### 2. Verify the development environment

```bash
node --version
npm --version
rustc --version
cargo --version
```

If Rust is installed through `rustup` but Cargo is missing from your `PATH`, add:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### 3. Install frontend dependencies

```bash
npm --prefix apps/desktop install
```

### 4. Launch the desktop application

```bash
cd apps/desktop

WEBKIT_DISABLE_DMABUF_RENDERER=1 \
WEBKIT_DISABLE_COMPOSITING_MODE=1 \
npm run tauri dev
```

The WebKit environment variables are compatibility workarounds for certain Linux graphics environments. If your setup doesn't require them, try running the development command without them.

> [!TIP]
> Run these commands from the repository root unless the instructions explicitly change directories.

---

## 🧪 Verification and Testing

Run checks before submitting changes or considering a cleanup complete.

### Frontend

From the `apps/desktop` directory:

```bash
npm test
npm run lint
npm run build
```

These commands respectively run the frontend test suite, check lint rules, and build the TypeScript/Vite frontend.

### Rust backend

From the `apps/desktop/src-tauri` directory:

```bash
cargo fmt --check
cargo check
cargo test
```

For an additional lint pass, where the project's installed toolchain supports it:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### ✅ Verification checklist

- [ ] Frontend tests pass.
- [ ] Frontend lint passes.
- [ ] Production frontend build succeeds.
- [ ] Rust formatting check passes.
- [ ] Rust compilation check passes.
- [ ] Rust tests pass.
- [ ] Clippy passes, if available and applicable.
- [ ] No unrelated files or pre-existing changes were overwritten.

> [!IMPORTANT]
> Commands listed here are verification instructions, not a claim that every check has passed on every platform or commit.

---

## 🔐 Security and Privacy

AI-Hub is designed around local application management rather than collecting credentials to act as an intermediary AI-chat service.

Its security-oriented principles include:

- **No credential scraping:** It should not extract login credentials from managed applications.
- **No unnecessary API-key collection:** Application lifecycle management should not require users to hand over AI-provider API keys.
- **Controlled process management:** Prefer targeted, graceful shutdown with timeout handling over indiscriminate process termination.
- **Explicit boundaries:** Keep desktop IPC commands, backend lifecycle logic, and platform runtime behavior separated.
- **Local-first operation:** Manage applications through the local desktop environment.

**Important distinction:** AI-Hub's local management architecture does not make the managed AI applications themselves offline-capable. Applications such as ChatGPT or Gemini may require an internet connection and their own authentication.

---

## 🗺️ Roadmap

The following are potential development directions, not promises that every item is already implemented.

- [ ] Improve application discovery and compatibility across desktop environments.
- [ ] Strengthen resource-aware lifecycle coordination.
- [ ] Expand and verify Windows runtime support.
- [ ] Improve error reporting and lifecycle observability.
- [ ] Expand automated tests for platform-specific behavior.
- [ ] Refine documentation for installation, development, and troubleshooting.

Have an idea? [Open a feature request](https://github.com/Darkwarrior10110/ai-hub/issues).

---

## 🤝 Contributing

Contributions, bug reports, and practical suggestions are welcome.

1. Fork the repository.
2. Create a focused feature branch.
3. Make your changes without overwriting unrelated work.
4. Run the relevant frontend and Rust checks.
5. Submit a pull request describing the change and verification results.

For bugs and feature requests, visit [GitHub Issues](https://github.com/Darkwarrior10110/ai-hub/issues).

---

## 📄 License

Check the repository for its current license before redistributing or incorporating AI-Hub into another project.

---

<p align="center">
  <strong>Built for practical AI workflows. Designed with resource-conscious computing in mind. ⚡</strong>
</p>

<p align="center">
  <sub>AI-Hub — One control center for the AI applications you already use.</sub>
</p>
