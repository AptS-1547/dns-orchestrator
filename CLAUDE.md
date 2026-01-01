# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DNS Orchestrator is a cross-platform DNS record management application (macOS, Windows, Linux, Android) supporting multiple DNS providers: Cloudflare, Alibaba Cloud DNS, Tencent Cloud DNSPod, and Huawei Cloud DNS.

**Tech Stack**: Tauri 2 (Rust) + React 19 + TypeScript 5 + Tailwind CSS 4 + Zustand 5

## Common Commands

```bash
# Development
pnpm tauri dev              # Desktop development with hot reload
pnpm tauri android dev      # Android development
pnpm dev:web                # Web frontend (requires actix-web backend)

# Build
pnpm tauri build            # Desktop production build
pnpm tauri android build    # Android production build
pnpm build:web              # Web frontend build

# Code Quality
pnpm lint                   # Biome lint for frontend
pnpm format:fix             # Biome format + lint fix
pnpm lint:rust              # Clippy for all Rust crates
pnpm format:rust            # cargo fmt for all Rust crates

# Version Management
pnpm sync-version           # Sync version to package.json, tauri.conf.json, Cargo.toml files

# Testing
cargo test -p dns-orchestrator-provider   # Provider library tests
cargo test --workspace                    # All Rust tests
```

## Architecture

```
Frontend                    Backend
─────────                   ───────
Components → Stores → Services → Transport → [Tauri IPC | HTTP] → Commands → Provider Library → DNS APIs
```

**Key Components**:

| Directory | Purpose |
|-----------|---------|
| `src/` | React frontend (components, stores, services, i18n) |
| `src/services/transport/` | Transport abstraction (`ITransport` interface) |
| `dns-orchestrator-provider/` | Standalone Rust crate with `DnsProvider` trait |
| `src-tauri/` | Tauri backend (commands, credentials, storage) |
| `src-actix-web/` | Web backend with SeaORM (WIP) |

**Transport Selection** (compile-time via vite.config.ts):
- Desktop/Mobile: `TauriTransport` (IPC)
- Web: `HttpTransport` (REST)

## Code Style

**Frontend (Biome)**:
- 2 spaces, double quotes, semicolons as needed, 100 char line width
- Components: `PascalCase.tsx`, Stores: `xxxStore.ts`, Services: `xxx.service.ts`

**Rust**:
- `cargo fmt` + `cargo clippy`
- `unsafe_code = "forbid"`, `unwrap_used = "warn"`, `expect_used = "warn"`

## Adding a New DNS Provider

1. Create `dns-orchestrator-provider/src/providers/your_provider.rs` implementing `DnsProvider` trait
2. Add feature flag in `dns-orchestrator-provider/Cargo.toml`
3. Register in `providers/mod.rs` and `factory.rs`
4. Add credentials variant to `ProviderCredentials` enum in `types.rs`
5. Add metadata in `factory.rs` `get_all_provider_metadata()`
6. Add translations in `src/i18n/locales/`

See `docs/DEVELOPMENT.md#adding-a-new-dns-provider` for detailed steps.

## Platform Notes

- **Android**: Uses `rustls` instead of `native-tls` (avoids OpenSSL cross-compilation)
- **Linux**: Requires DBus Secret Service (GNOME Keyring or KWallet) for credential storage
- **Backend hot reload**: Frontend changes instant, Rust changes require restart of `pnpm tauri dev`

## Debugging

```bash
RUST_LOG=debug pnpm tauri dev           # Enable debug logging
RUST_LOG=dns_orchestrator=trace pnpm tauri dev  # Verbose logging
```

DevTools: `Cmd+Option+I` (macOS) / `Ctrl+Shift+I` (Linux) / `F12` (Windows)
