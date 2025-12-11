# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DNS Orchestrator is a cross-platform DNS record management application supporting Cloudflare, Aliyun, DNSPod, and Huaweicloud providers. It has three deployment targets:
- **Desktop (Tauri)**: Primary platform via `src-tauri/`
- **Web (Actix-web)**: Backend API via `src-actix-web/` (in development)
- **Mobile (Android)**: Via Tauri Android target

## Commands

```bash
# Development
pnpm install                    # Install frontend dependencies
pnpm tauri dev                  # Start Tauri desktop development
pnpm dev:web                    # Start web-only frontend (VITE_PLATFORM=web)

# Build
pnpm tauri build               # Build production desktop app
pnpm build:web                 # Build web frontend only

# Code Quality
pnpm check                     # Run all checks (lint + format for both TS and Rust)
pnpm lint                      # Lint TypeScript (Biome)
pnpm lint:fix                  # Fix linting issues
pnpm format                    # Format TypeScript
pnpm lint:rust                 # Run clippy on Rust code
pnpm format:rust               # Format Rust code

# Rust Only
cargo build --manifest-path=src-tauri/Cargo.toml    # Build Tauri backend
cargo test --manifest-path=src-tauri/Cargo.toml     # Run Rust tests
cargo clippy --manifest-path=src-tauri/Cargo.toml --all-targets --all-features -- -D warnings

# Version Management
pnpm sync-version              # Sync version across package.json, tauri.conf.json, Cargo.toml
```

## Architecture

### Frontend (React + TypeScript)

```
src/
├── components/           # React components by feature domain
│   ├── account/         # Account management (AccountForm, AccountList, etc.)
│   ├── dns/             # DNS record CRUD (DnsRecordTable, DnsRecordForm, etc.)
│   ├── domain/          # Domain listing
│   ├── toolbox/         # Network utilities (DNS/WHOIS/SSL lookup)
│   └── ui/              # Radix UI component wrappers
├── stores/              # Zustand stores (accountStore, dnsStore, domainStore, etc.)
├── services/            # Service layer with transport abstraction
│   └── transport/       # Platform-agnostic transport (Tauri IPC or HTTP)
├── types/               # TypeScript interfaces matching Rust types
└── i18n/locales/        # en-US.ts and zh-CN.ts translations
```

**Key Patterns:**
- **Transport Abstraction**: `src/services/transport/` allows switching between Tauri IPC (`tauri.transport.ts`) and HTTP (`http.transport.ts`) via Vite alias at compile time
- **State Management**: Zustand stores per domain (account, dns, domain, toolbox, settings)
- **Type Safety**: Frontend types in `src/types/` must match Rust `src-tauri/src/types.rs`

### Backend (Rust/Tauri)

```
src-tauri/src/
├── commands/            # Tauri command handlers (account.rs, dns.rs, domain.rs, toolbox.rs)
├── providers/           # DNS provider implementations
│   ├── mod.rs          # DnsProvider trait + ProviderRegistry + create_provider()
│   ├── cloudflare.rs
│   ├── aliyun.rs
│   ├── dnspod.rs
│   └── huaweicloud.rs
├── credentials/         # System keychain integration (keychain.rs, android.rs)
├── storage/             # Account metadata persistence
├── crypto.rs            # AES-GCM encryption for account export
├── error.rs             # Error types (DnsError, ProviderError)
├── types.rs             # Shared data structures
└── lib.rs               # AppState, setup(), Tauri plugin registration
```

**Key Patterns:**
- **Provider Abstraction**: All providers implement `DnsProvider` trait with methods: `validate_credentials()`, `list_domains()`, `list_records()`, `create_record()`, `update_record()`, `delete_record()`
- **Registry Pattern**: `ProviderRegistry` manages provider instances by `account_id`
- **Error Mapping**: `ProviderErrorMapper` trait standardizes provider-specific API errors
- **Credential Security**: Uses `keyring` crate for system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)

### Actix-web Backend (src-actix-web/)

Alternative web backend using SeaORM for database access. Currently in early development.

## Adding a New DNS Provider

1. Create `src-tauri/src/providers/your_provider.rs` implementing `DnsProvider` trait
2. Register in `src-tauri/src/providers/mod.rs`:
   - Add module and re-export
   - Add to `create_provider()` match
   - Add metadata to `get_all_provider_metadata()`
3. Add provider type to `src/types/provider.ts`
4. Add icon in `src/components/account/ProviderIcon.tsx`
5. Add translations in `src/i18n/locales/`

## Key Files

- `src-tauri/src/providers/mod.rs`: `DnsProvider` trait definition, provider factory, metadata
- `src-tauri/src/lib.rs`: `AppState` struct, Tauri setup, plugin registration
- `src/services/transport/types.ts`: `CommandMap` defining all Tauri commands and their signatures
- `src/stores/`: All Zustand stores for state management

## Platform Differences

- **Desktop/Android**: Uses Tauri IPC via `@tauri-apps/api`
- **Web**: Uses HTTP transport to actix-web backend
- Platform detection via `VITE_PLATFORM` env var and `src/lib/env.ts`
- Android uses `tauri-plugin-stronghold` instead of keyring for credential storage

## Code Style

- **TypeScript**: Biome for linting and formatting
- **Rust**: Clippy (pedantic) + rustfmt, `unsafe_code = "forbid"`
- Clippy warnings treated as errors in CI: `unwrap_used`, `expect_used`, `panic`
