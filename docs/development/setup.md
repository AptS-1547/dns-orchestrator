# Development Guide

This guide will help you set up your development environment and understand the codebase structure for contributing to DNS Orchestrator.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Adding a New DNS Provider](#adding-a-new-dns-provider)
- [Building for Different Platforms](#building-for-different-platforms)
- [Testing](#testing)
- [Common Issues](#common-issues)

## Prerequisites

### Required Tools

- **Node.js**: 22+ (LTS recommended)
- **pnpm**: 10+ (package manager)
- **Rust**: Latest stable version (install via [rustup](https://rustup.rs/))
- **Git**: For version control

### Platform-Specific Dependencies

#### macOS
```bash
xcode-select --install
```

#### Windows
Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with C++ development tools.

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  xdg-utils \
  build-essential \
  curl \
  wget
```

#### Android Development
```bash
# Install Android SDK and NDK via Android Studio or command line
# Set environment variables
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/<version>

# Initialize Tauri Android
pnpm tauri android init
```

For other distributions, see [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/).

## Getting Started

### Clone the Repository

```bash
git clone https://github.com/AptS-1547/dns-orchestrator.git
cd dns-orchestrator
```

### Install Dependencies

```bash
# Install frontend dependencies
pnpm install

# Rust dependencies are managed by Cargo and will be installed on first build
```

### Start Development Server

```bash
# Desktop: Start Tauri in development mode with hot reload
pnpm tauri dev

# Android: Start Android development
pnpm tauri android dev

# Web mode: Start frontend with HTTP transport (requires actix-web backend)
pnpm dev:web
```

This will:
1. Start the Vite development server for the React frontend
2. Compile the Rust backend
3. Launch the application window with hot reload enabled

### Build for Production

```bash
# Desktop build
pnpm tauri build

# Android build
pnpm tauri android build

# Web frontend build
pnpm build:web
```

Built artifacts will be in `src-tauri/target/release/bundle/`.

## Project Structure

```
dns-orchestrator/
├── src/                              # Frontend (React + TypeScript)
│   ├── components/                   # React components by feature
│   │   ├── account/                  # Account management UI
│   │   ├── dns/                      # DNS record management
│   │   ├── domain/                   # Domain management
│   │   ├── domains/                  # Domain selector page
│   │   ├── home/                     # Home dashboard
│   │   ├── toolbox/                  # Network toolbox
│   │   ├── settings/                 # Settings page
│   │   ├── layout/                   # Layout components
│   │   ├── navigation/               # Navigation components
│   │   ├── error/                    # Error boundary
│   │   └── ui/                       # Reusable UI components
│   ├── services/                     # Service layer
│   │   ├── transport/                # Transport abstraction
│   │   │   ├── types.ts              # ITransport, CommandMap
│   │   │   ├── tauri.transport.ts    # Tauri IPC implementation
│   │   │   └── http.transport.ts     # HTTP REST implementation
│   │   ├── account.service.ts
│   │   ├── dns.service.ts
│   │   ├── domain.service.ts
│   │   └── toolbox.service.ts
│   ├── stores/                       # Zustand state management
│   ├── types/                        # TypeScript type definitions
│   ├── i18n/                         # Internationalization
│   ├── lib/                          # Utility functions
│   ├── constants/                    # Application constants
│   └── hooks/                        # Custom React hooks
│
├── dns-orchestrator-provider/        # Standalone Provider Library
│   ├── src/
│   │   ├── lib.rs                    # Library entry, re-exports
│   │   ├── traits.rs                 # DnsProvider trait
│   │   ├── types.rs                  # Domain, DnsRecord, etc.
│   │   ├── error.rs                  # ProviderError enum
│   │   ├── factory.rs                # create_provider(), metadata
│   │   └── providers/                # Provider implementations
│   │       ├── mod.rs
│   │       ├── cloudflare.rs
│   │       ├── aliyun.rs
│   │       ├── dnspod.rs
│   │       └── huaweicloud.rs
│   └── Cargo.toml
│
├── src-tauri/                        # Tauri Backend (Desktop/Mobile)
│   ├── src/
│   │   ├── commands/                 # Tauri command handlers
│   │   │   ├── mod.rs
│   │   │   ├── account.rs
│   │   │   ├── dns.rs
│   │   │   ├── domain.rs
│   │   │   ├── toolbox.rs
│   │   │   └── updater.rs
│   │   ├── providers/                # Provider registry
│   │   │   └── mod.rs                # ProviderRegistry + re-exports
│   │   ├── credentials/              # Credential storage
│   │   │   ├── mod.rs
│   │   │   ├── keychain.rs           # Desktop keychain
│   │   │   └── android.rs            # Android Stronghold
│   │   ├── storage/                  # Local data persistence
│   │   ├── crypto.rs                 # Encryption utilities
│   │   ├── error.rs                  # Error types
│   │   ├── types.rs                  # Rust type definitions
│   │   ├── lib.rs                    # Tauri library entry
│   │   └── main.rs                   # Application entry
│   ├── tauri-plugin-apk-installer/   # Android APK installer plugin
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src-actix-web/                    # Web Backend (WIP)
│   ├── src/main.rs                   # Actix-web server entry
│   ├── migration/                    # SeaORM migrations
│   └── Cargo.toml
│
├── scripts/
│   └── sync-version.mjs              # Version sync script
├── package.json
├── vite.config.ts                    # Platform-aware Vite config
└── tsconfig.json
```

### Key Architecture Components

#### Frontend
- **Services**: Abstract backend communication via `ITransport` interface
- **Transport**: Tauri IPC for desktop/mobile, HTTP for web
- **Stores**: Zustand stores for state management (one per feature)
- **Components**: Feature-based organization

#### Backend (Tauri)
- **Commands**: Tauri command handlers exposed to frontend
- **Providers**: Re-exports from `dns-orchestrator-provider` + `ProviderRegistry`
- **Credentials**: Platform-specific secure storage

#### Provider Library
- **Standalone crate**: Reusable across Tauri and web backends
- **Feature flags**: Enable providers and TLS backends selectively
- **Unified errors**: `ProviderError` for all provider-specific errors

## Development Workflow

### Hot Reload

- **Frontend changes**: Instant reload without losing state
- **Backend changes**: Requires manual restart of `pnpm tauri dev`
- **Provider library changes**: Requires restart

### Debugging

#### Frontend Debugging
Open DevTools in the application window:
- **macOS/Linux**: `Cmd+Option+I` or `Ctrl+Shift+I`
- **Windows**: `F12`

#### Backend Debugging
```bash
# Enable debug logging
RUST_LOG=debug pnpm tauri dev

# More verbose
RUST_LOG=dns_orchestrator=trace pnpm tauri dev
```

### Version Synchronization

```bash
pnpm sync-version
```

This updates version in:
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `src-actix-web/Cargo.toml`

Always run this before creating a release.

### Code Quality

```bash
# Frontend
pnpm lint          # Run Biome linter
pnpm format:fix    # Format code

# Backend
pnpm lint:rust     # Run Clippy
pnpm format:rust   # Format Rust code

# All checks
pnpm check
```

## Adding a New DNS Provider

Since v1.1.0, providers are implemented in the standalone `dns-orchestrator-provider` library.

### Step 1: Create Provider Implementation

Create `dns-orchestrator-provider/src/providers/your_provider.rs`:

```rust
use async_trait::async_trait;
use reqwest::Client;

use crate::error::{ProviderError, Result};
use crate::traits::DnsProvider;
use crate::types::*;

pub struct YourProvider {
    client: Client,
    api_key: String,
}

impl YourProvider {
    pub fn new(credentials: ProviderCredentials) -> Result<Self> {
        let ProviderCredentials::YourProvider { api_key } = credentials else {
            return Err(ProviderError::InvalidCredentials {
                provider: "your_provider".to_string(),
            });
        };

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    fn provider_name() -> &'static str {
        "your_provider"
    }
}

#[async_trait]
impl DnsProvider for YourProvider {
    async fn validate_credentials(&self) -> Result<()> {
        // Make a simple API call to verify credentials
        todo!()
    }

    async fn list_domains(&self, params: &PaginationParams) -> Result<PaginatedResponse<Domain>> {
        todo!()
    }

    async fn get_domain(&self, domain_id: &str) -> Result<Domain> {
        todo!()
    }

    async fn list_records(
        &self,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> Result<PaginatedResponse<DnsRecord>> {
        todo!()
    }

    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord> {
        todo!()
    }

    async fn update_record(
        &self,
        record_id: &str,
        req: &UpdateDnsRecordRequest,
    ) -> Result<DnsRecord> {
        todo!()
    }

    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()> {
        todo!()
    }
}
```

### Step 2: Add Feature Flag

Update `dns-orchestrator-provider/Cargo.toml`:

```toml
[features]
your_provider = []
all-providers = ["cloudflare", "aliyun", "dnspod", "huaweicloud", "your_provider"]
```

### Step 3: Register Provider

Update `dns-orchestrator-provider/src/providers/mod.rs`:

```rust
#[cfg(feature = "your_provider")]
mod your_provider;
#[cfg(feature = "your_provider")]
pub use your_provider::YourProvider;
```

Update `dns-orchestrator-provider/src/factory.rs`:

```rust
pub fn create_provider(credentials: ProviderCredentials) -> Result<Arc<dyn DnsProvider>> {
    match &credentials {
        // ... existing providers
        #[cfg(feature = "your_provider")]
        ProviderCredentials::YourProvider { .. } => {
            Ok(Arc::new(YourProvider::new(credentials)?))
        }
    }
}
```

### Step 4: Add Credentials Type

Update `dns-orchestrator-provider/src/types.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderCredentials {
    // ... existing variants
    YourProvider { api_key: String },
}

// Add to ProviderType enum
pub enum ProviderType {
    // ...
    YourProvider,
}
```

### Step 5: Add Provider Metadata

Update `dns-orchestrator-provider/src/factory.rs`:

```rust
pub fn get_all_provider_metadata() -> Vec<ProviderMetadata> {
    vec![
        // ... existing providers
        #[cfg(feature = "your_provider")]
        ProviderMetadata {
            id: "your_provider".to_string(),
            name: "Your Provider".to_string(),
            description: "Your DNS provider description".to_string(),
            required_fields: vec![
                ProviderCredentialField {
                    key: "api_key".to_string(),
                    label: "API Key".to_string(),
                    field_type: FieldType::Password,
                    placeholder: Some("Enter API Key".to_string()),
                    help_text: Some("Get this from your provider dashboard".to_string()),
                },
            ],
            features: ProviderFeatures::default(),
        },
    ]
}
```

### Step 6: Add Frontend Translations

**`src/i18n/locales/en-US.ts`:**
```typescript
providers: {
  your_provider: 'Your Provider',
}
```

**`src/i18n/locales/zh-CN.ts`:**
```typescript
providers: {
  your_provider: '你的服务商',
}
```

### Step 7: Add Provider Icon (Optional)

Update `src/components/account/ProviderIcon.tsx` if you have a custom icon.

### Step 8: Test

```bash
# Run provider library tests
cargo test -p dns-orchestrator-provider

# Start development server and test UI
pnpm tauri dev
```

## Building for Different Platforms

### Desktop (macOS, Windows, Linux)

```bash
pnpm tauri build
```

### Android

```bash
# Initialize (first time only)
pnpm tauri android init

# Development
pnpm tauri android dev

# Release build
pnpm tauri android build
```

**Note**: Android uses `rustls` instead of `native-tls` to avoid OpenSSL cross-compilation issues.

### Web Mode

```bash
# Development (requires running actix-web backend)
pnpm dev:web

# Build
pnpm build:web
```

### GitHub Actions Release

Push a tag to trigger automated builds:

```bash
git tag v1.1.0
git push origin v1.1.0
```

**Supported platforms:**
- macOS (Apple Silicon + Intel)
- Windows (x64 + ARM64)
- Linux (x64 + ARM64)
- Android (ARM64, ARM32, x64)

## Testing

### Running Tests

```bash
# Provider library tests
cargo test -p dns-orchestrator-provider

# Tauri backend tests
cargo test -p dns-orchestrator

# All Rust tests
cargo test --workspace
```

### Manual Testing Checklist

Before releasing:

- [ ] Account creation for all providers
- [ ] Credential validation (valid and invalid)
- [ ] Domain listing with pagination
- [ ] DNS record CRUD operations
- [ ] Search and filtering
- [ ] Account import/export with encryption
- [ ] DNS lookup tool
- [ ] WHOIS lookup tool
- [ ] Theme switching
- [ ] Language switching
- [ ] Android build and basic functionality

## Common Issues

### Build Errors

**Issue**: `webkit2gtk` not found (Linux)
```bash
sudo apt-get install libwebkit2gtk-4.1-dev
```

**Issue**: OpenSSL errors on Android
```bash
# Ensure using rustls feature for Android
# Check src-tauri/Cargo.toml has default-features = false for Android target
```

**Issue**: Rust linker errors
```bash
rustup update stable
cargo clean
```

**Issue**: pnpm installation fails
```bash
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

### Runtime Errors

**Issue**: "Failed to load credentials"
- Ensure system keychain service is running (Linux: `gnome-keyring` or `kwallet`)
- On Android, ensure Stronghold is properly initialized

**Issue**: Provider API errors
- Check API credentials are correct
- Enable debug logging: `RUST_LOG=debug pnpm tauri dev`

### Development Tips

1. **Use React DevTools**: Inspect Zustand stores and component state
2. **Check Rust logs**: Backend errors are logged to console in dev mode
3. **Test with real credentials**: Use test/sandbox API keys when available
4. **Incremental compilation**: Keep `pnpm tauri dev` running for faster iteration
5. **Clean build if weird errors**: `cargo clean && pnpm tauri dev`

## Getting Help

- **Documentation**: [Tauri Docs](https://v2.tauri.app/), [React Docs](https://react.dev/)
- **Issues**: [GitHub Issues](https://github.com/AptS-1547/dns-orchestrator/issues)
- **Discussions**: [GitHub Discussions](https://github.com/AptS-1547/dns-orchestrator/discussions)

---

Happy coding! 🚀
