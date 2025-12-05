# Development Guide

This guide will help you set up your development environment and understand the codebase structure for contributing to DNS Orchestrator.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Adding a New DNS Provider](#adding-a-new-dns-provider)
- [Building and Release](#building-and-release)
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

For other distributions, see [Tauri Prerequisites](https://tauri.app/v2/guides/prerequisites/).

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
# Start Tauri in development mode with hot reload
pnpm tauri dev
```

This will:
1. Start the Vite development server for the React frontend
2. Compile the Rust backend
3. Launch the application window with hot reload enabled

### Build for Production

```bash
# Build optimized production bundle
pnpm tauri build
```

Built artifacts will be in `src-tauri/target/release/bundle/`.

## Project Structure

```
dns-orchestrator/
├── src/                          # Frontend (React + TypeScript)
│   ├── components/               # React components
│   │   ├── account/              # Account management UI
│   │   ├── dns/                  # DNS record management
│   │   ├── domain/               # Domain management
│   │   ├── toolbox/              # Network toolbox (DNS/WHOIS)
│   │   ├── settings/             # Settings page
│   │   └── ui/                   # Reusable UI components
│   ├── stores/                   # Zustand state management
│   │   ├── accountStore.ts       # Account state
│   │   ├── dnsStore.ts           # DNS records state
│   │   ├── domainStore.ts        # Domain state
│   │   ├── toolboxStore.ts       # Toolbox state
│   │   └── settingsStore.ts      # App settings
│   ├── types/                    # TypeScript type definitions
│   │   ├── account.ts
│   │   ├── dns.ts
│   │   ├── domain.ts
│   │   ├── provider.ts
│   │   └── toolbox.ts
│   ├── i18n/                     # Internationalization
│   │   ├── index.ts
│   │   └── locales/
│   │       ├── en-US.ts          # English translations
│   │       └── zh-CN.ts          # Chinese translations
│   ├── App.tsx                   # Root component
│   ├── main.tsx                  # React entry point
│   └── index.css                 # Global styles
│
├── src-tauri/                    # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── commands/             # Tauri command handlers
│   │   │   ├── account.rs        # Account management commands
│   │   │   ├── dns.rs            # DNS operations
│   │   │   ├── domain.rs         # Domain operations
│   │   │   └── toolbox.rs        # Network toolbox commands
│   │   ├── providers/            # DNS provider implementations
│   │   │   ├── mod.rs            # Provider trait & registry
│   │   │   ├── cloudflare.rs
│   │   │   ├── aliyun.rs
│   │   │   ├── dnspod.rs
│   │   │   └── huaweicloud.rs
│   │   ├── credentials/          # Secure credential storage
│   │   │   ├── mod.rs
│   │   │   └── keychain.rs       # System keychain integration
│   │   ├── storage/              # Local data persistence
│   │   │   ├── mod.rs
│   │   │   └── account_store.rs
│   │   ├── crypto.rs             # Encryption utilities
│   │   ├── error.rs              # Error types and handling
│   │   ├── types.rs              # Rust type definitions
│   │   ├── lib.rs                # Tauri library entry
│   │   └── main.rs               # Application entry
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri configuration
│   └── build.rs                  # Build script
│
├── .github/
│   └── workflows/
│       └── release.yml           # GitHub Actions release workflow
├── package.json                  # Frontend dependencies & scripts
├── vite.config.ts                # Vite configuration
├── tsconfig.json                 # TypeScript configuration
└── README.md
```

### Key Components

#### Frontend
- **Components**: Organized by feature (account, dns, domain, toolbox)
- **Stores**: Zustand stores for state management (one per feature domain)
- **Types**: Shared TypeScript interfaces matching Rust backend types
- **i18n**: Translation files for English and Chinese

#### Backend
- **Commands**: Tauri command handlers exposed to frontend via `invoke()`
- **Providers**: DNS provider implementations following the `DnsProvider` trait
- **Credentials**: System keychain integration for secure storage
- **Storage**: Local JSON-based account metadata storage

## Development Workflow

### Hot Reload

The development server supports hot module replacement (HMR):
- **Frontend changes**: Instant reload without losing state
- **Backend changes**: Requires manual restart of `pnpm tauri dev`

### Debugging

#### Frontend Debugging
Open DevTools in the application window:
- **macOS/Linux**: `Cmd+Option+I` or `Ctrl+Shift+I`
- **Windows**: `F12`

#### Backend Debugging
Add logging with the `log` crate:

```rust
use log::{info, warn, error};

info!("This is an info message");
warn!("This is a warning");
error!("This is an error");
```

Run with logging enabled:
```bash
RUST_LOG=debug pnpm tauri dev
```

### Version Synchronization

The project uses a custom script to keep versions in sync:

```bash
pnpm sync-version
```

This updates:
- `package.json` → `version`
- `src-tauri/tauri.conf.json` → `version`
- `src-tauri/Cargo.toml` → `version`

Always run this before creating a release.

## Adding a New DNS Provider

This section guides you through adding support for a new DNS provider.

### Step 1: Create the Provider Implementation

Create a new file in `src-tauri/src/providers/your_provider.rs`:

```rust
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::error::{DnsError, Result};
use crate::providers::DnsProvider;
use crate::types::*;

pub struct YourProvider {
    client: Client,
    credentials: HashMap<String, String>,
}

impl YourProvider {
    pub fn new(credentials: HashMap<String, String>) -> Self {
        Self {
            client: Client::new(),
            credentials,
        }
    }

    fn get_credential(&self, key: &str) -> Result<String> {
        self.credentials
            .get(key)
            .cloned()
            .ok_or_else(|| DnsError::MissingCredential(key.to_string()))
    }
}

#[async_trait]
impl DnsProvider for YourProvider {
    fn id(&self) -> &'static str {
        "your_provider"
    }

    async fn validate_credentials(&self) -> Result<bool> {
        // Implement credential validation
        // Make a simple API call to verify credentials work
        todo!()
    }

    async fn list_domains(&self, params: &PaginationParams) -> Result<PaginatedResponse<Domain>> {
        // Implement domain listing with pagination
        todo!()
    }

    async fn get_domain(&self, domain_id: &str) -> Result<Domain> {
        // Implement getting single domain details
        todo!()
    }

    async fn list_records(
        &self,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> Result<PaginatedResponse<DnsRecord>> {
        // Implement DNS record listing with pagination and filtering
        todo!()
    }

    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord> {
        // Implement DNS record creation
        todo!()
    }

    async fn update_record(
        &self,
        record_id: &str,
        req: &UpdateDnsRecordRequest,
    ) -> Result<DnsRecord> {
        // Implement DNS record update
        todo!()
    }

    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()> {
        // Implement DNS record deletion
        todo!()
    }
}
```

### Step 2: Register the Provider

Update `src-tauri/src/providers/mod.rs`:

```rust
mod your_provider;
pub use your_provider::YourProvider;

// In create_provider function:
pub fn create_provider(
    provider_type: &str,
    credentials: HashMap<String, String>,
) -> Result<Arc<dyn DnsProvider>> {
    match provider_type {
        "cloudflare" => Ok(Arc::new(CloudflareProvider::new(credentials))),
        "aliyun" => Ok(Arc::new(AliyunProvider::new(credentials))),
        "dnspod" => Ok(Arc::new(DnspodProvider::new(credentials))),
        "huaweicloud" => Ok(Arc::new(HuaweicloudProvider::new(credentials))),
        "your_provider" => Ok(Arc::new(YourProvider::new(credentials))), // Add this line
        _ => Err(DnsError::ProviderNotFound(provider_type.to_string())),
    }
}

// Add provider metadata in get_all_provider_metadata():
ProviderMetadata {
    id: "your_provider".to_string(),
    name: "Your Provider".to_string(),
    description: "Description of your DNS provider".to_string(),
    required_fields: vec![
        ProviderCredentialField {
            key: "apiKey".to_string(),
            label: "API Key".to_string(),
            field_type: "password".to_string(),
            placeholder: Some("Enter API Key".to_string()),
            help_text: Some("Get this from your provider dashboard".to_string()),
        }
    ],
    features: ProviderFeatures::default(),
},
```

### Step 3: Add Frontend Types

Update `src/types/provider.ts`:

```typescript
export type ProviderType =
  | 'cloudflare'
  | 'aliyun'
  | 'dnspod'
  | 'huaweicloud'
  | 'your_provider';  // Add this line
```

### Step 4: Add UI Icon

Update `src/components/account/ProviderIcon.tsx`:

```tsx
const providerIcons: Record<ProviderType, React.ReactNode> = {
  // ... existing providers
  your_provider: <YourProviderIcon className="w-5 h-5" />,
};
```

### Step 5: Add Translations

Update translation files:

**`src/i18n/locales/en-US.ts`:**
```typescript
providers: {
  // ... existing providers
  your_provider: 'Your Provider',
}
```

**`src/i18n/locales/zh-CN.ts`:**
```typescript
providers: {
  // ... existing providers
  your_provider: '你的服务商',
}
```

### Step 6: Test Your Provider

1. Start the development server: `pnpm tauri dev`
2. Add a new account with your provider
3. Test all operations: list domains, list records, create/update/delete records
4. Verify pagination and search functionality

### Reference Implementations

For complete examples, see:
- **Simple provider**: `src-tauri/src/providers/cloudflare.rs`
- **Complex provider**: `src-tauri/src/providers/aliyun.rs`

## Building and Release

### Local Build

```bash
# Development build (faster, with debug info)
cargo build --manifest-path=src-tauri/Cargo.toml

# Production build (optimized)
pnpm tauri build
```

### Version Management

Before releasing:

1. Update version in `package.json`
2. Run `pnpm sync-version` to sync to other files
3. Commit changes: `git commit -am "chore: bump version to x.y.z"`
4. Create git tag: `git tag vx.y.z`
5. Push: `git push && git push --tags`

### GitHub Actions Release

The project uses GitHub Actions for automated releases (`.github/workflows/release.yml`).

**Supported Platforms:**
- macOS (Apple Silicon + Intel)
- Windows (x64 + ARM64)
- Linux (x64 + ARM64)

**To trigger a release:**

```bash
git tag v1.0.0
git push origin v1.0.0
```

The workflow will:
1. Build for all platforms in parallel
2. Sign the binaries (requires `TAURI_SIGNING_PRIVATE_KEY` secret)
3. Create a GitHub Release draft
4. Upload all installers and update manifests

## Testing

### Running Tests

```bash
# Run Rust tests
cargo test --manifest-path=src-tauri/Cargo.toml

# Run frontend tests (if you add them)
pnpm test
```

### Manual Testing Checklist

Before releasing, manually test:

- [ ] Account creation for all providers
- [ ] Credential validation (valid and invalid credentials)
- [ ] Domain listing with pagination
- [ ] DNS record CRUD operations
- [ ] Search and filtering functionality
- [ ] Account import/export with encryption
- [ ] DNS lookup tool
- [ ] WHOIS lookup tool
- [ ] Theme switching
- [ ] Language switching
- [ ] Application updates (if update server is configured)

## Common Issues

### Build Errors

**Issue**: `webkit2gtk` not found (Linux)
```bash
sudo apt-get install libwebkit2gtk-4.1-dev
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

**Issue**: CORS errors in development
- The Tauri app uses the custom protocol `tauri://localhost` which bypasses CORS

**Issue**: Provider API errors
- Check API credentials are correct
- Verify API endpoints are accessible (check firewall/proxy)
- Enable debug logging: `RUST_LOG=debug pnpm tauri dev`

### Development Tips

1. **Use the React DevTools**: Inspect Zustand stores and component state
2. **Check Rust logs**: Backend errors are logged to console in dev mode
3. **Test with real credentials**: Use test/sandbox API keys when available
4. **Incremental compilation**: Keep `pnpm tauri dev` running for faster iteration
5. **Clean build if weird errors**: `cargo clean && pnpm tauri dev`

## Getting Help

- **Documentation**: [Tauri Docs](https://tauri.app/), [React Docs](https://react.dev/)
- **Issues**: [GitHub Issues](https://github.com/AptS-1547/dns-orchestrator/issues)
- **Discussions**: [GitHub Discussions](https://github.com/AptS-1547/dns-orchestrator/discussions)

## Contributing

See the [Contributing section](../README.md#contributing) in the main README for guidelines.

---

Happy coding! 🚀
