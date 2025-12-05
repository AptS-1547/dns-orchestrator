# DNS Orchestrator

![GitHub release (latest by date)](https://img.shields.io/github/v/release/AptS-1547/dns-orchestrator)
![Release Workflow](https://github.com/AptS-1547/dns-orchestrator/actions/workflows/release.yml/badge.svg)
![License](https://img.shields.io/github/license/AptS-1547/dns-orchestrator)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue)

A cross-platform desktop application for unified management of DNS records across multiple DNS service providers.

[简体中文](./README.zh-CN.md) | English

## Features

- **Multi-Account Management** - Manage multiple DNS provider accounts with secure credential storage
- **Universal DNS Management** - Create, read, update, and delete DNS records across all providers
- **Advanced Search & Filtering** - Pagination, real-time search, and record type filtering with infinite scroll
- **Account Import/Export** - Encrypted backup and migration of account configurations
- **Network Toolbox** - Built-in DNS lookup and WHOIS query tools with history tracking
- **Cross-Platform** - Native desktop experience on macOS, Windows, and Linux
- **Modern UI** - Clean interface with dark/light theme support and bilingual (English/Chinese) localization

## Supported DNS Providers

| Provider | Features |
|----------|----------|
| **Cloudflare** | Full DNS management, CDN proxy toggle support |
| **Alibaba Cloud DNS** | Comprehensive record management with pagination and filtering |
| **Tencent Cloud DNSPod** | Complete DNS operations with search capabilities |
| **Huawei Cloud DNS** | Full-featured DNS management with type filtering |

## Quick Start

### Download

Download the latest version for your platform from the [Releases](https://github.com/AptS-1547/dns-orchestrator/releases) page:

- **macOS**: `.dmg` (Apple Silicon / Intel)
- **Windows**: `.msi` or `.exe` (x64, ARM64)
- **Linux**: `.deb` or `.AppImage` (x64, ARM64)

### Installation

#### macOS
1. Download the `.dmg` file for your architecture
2. Open the `.dmg` and drag DNS Orchestrator to Applications
3. Launch from Applications (you may need to approve it in System Preferences → Security & Privacy)

#### Windows
1. Download the `.msi` installer
2. Run the installer and follow the setup wizard
3. Launch DNS Orchestrator from the Start Menu

#### Linux
1. Download the `.deb` package or `.AppImage`
2. For `.deb`: `sudo dpkg -i dns-orchestrator_*.deb`
3. For `.AppImage`: Make executable (`chmod +x`) and run directly

### First Use

1. Click **"Add Account"** to configure your first DNS provider
2. Select provider type and enter your API credentials
3. View and manage your domains and DNS records
4. Use the **Network Toolbox** for DNS queries and WHOIS lookups

## Core Functionality

### Account Management
- Add unlimited accounts from multiple providers
- Secure credential storage using system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Import/export accounts with encryption for backup and migration
- Easy switching between accounts

### Domain Management
- Browse all domains across providers with pagination
- Infinite scroll for large domain lists
- Quick domain selection and filtering

### DNS Record Management
- **Supported Record Types**: A, AAAA, CNAME, MX, TXT, NS, SRV, CAA
- **Pagination**: Efficient loading with 20 records per page
- **Real-time Search**: Instant filtering with debounced search
- **Type Filtering**: Filter by record type for focused management
- **Bulk Operations**: Create, update, and delete records with validation
- **Cloudflare CDN Proxy**: Toggle proxy status for A/AAAA/CNAME records

### Network Toolbox
- **DNS Lookup**: Query DNS records (A, AAAA, CNAME, MX, TXT, NS, SOA, SRV, CAA, PTR, ALL)
- **WHOIS Query**: Retrieve domain registration information
- **History Tracking**: Quick access to recent queries

### Themes & Localization
- **Themes**: Light and dark mode with system preference detection
- **Languages**: English and Simplified Chinese
- Seamless language switching without restart

## Tech Stack

### Frontend
- **Framework**: React 19 + TypeScript 5
- **UI**: Tailwind CSS 4 + Radix UI components
- **State Management**: Zustand 5
- **Build Tool**: Vite 7
- **Internationalization**: i18next + react-i18next
- **Icons**: Lucide React

### Backend
- **Framework**: Tauri 2 (Rust)
- **Runtime**: Tokio (async runtime)
- **HTTP Client**: Reqwest
- **Credential Storage**: keyring 3 (system keychain integration)
- **Encryption**: Built-in crypto module for account export

### Security
- API credentials stored in system keychain, never in plaintext
- Encrypted account import/export with password protection
- Secure HTTPS communication with DNS providers

## Development

### Prerequisites
- Node.js 22+ and pnpm 10+
- Rust (latest stable)
- Platform-specific dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: MSVC (Visual Studio Build Tools)
  - **Linux**: webkit2gtk, libappindicator, librsvg, patchelf

### Setup

```bash
# Clone repository
git clone https://github.com/AptS-1547/dns-orchestrator.git
cd dns-orchestrator

# Install dependencies
pnpm install

# Start development mode
pnpm tauri dev

# Build for production
pnpm tauri build

# Sync version across package.json, tauri.conf.json, and Cargo.toml
pnpm sync-version
```

For detailed development instructions, see [DEVELOPMENT.md](./docs/DEVELOPMENT.md).

## Architecture

DNS Orchestrator follows a clean architecture pattern:

- **Frontend**: React components communicate via Zustand stores
- **Backend**: Rust-based Tauri commands handle business logic
- **Abstraction**: `DnsProvider` trait enables easy addition of new providers
- **Registry**: Dynamic provider management through `ProviderRegistry`

For in-depth architectural details, see [ARCHITECTURE.md](./docs/ARCHITECTURE.md).

## System Requirements

- **macOS**: 10.13 (High Sierra) or later
- **Windows**: 10 or later
- **Linux**: Modern distribution with DBus Secret Service (GNOME Keyring, KWallet, etc.)

## Contributing

Contributions are welcome! Here's how you can help:

1. **Report Bugs**: Open an issue with reproduction steps
2. **Suggest Features**: Share your ideas in the issues
3. **Add DNS Providers**: Follow the guide in [DEVELOPMENT.md](./docs/DEVELOPMENT.md#adding-a-new-dns-provider)
4. **Improve Translations**: Update locale files in `src/i18n/locales/`
5. **Submit Pull Requests**: Fork, branch, code, and PR

Please ensure your code follows the existing style and includes appropriate error handling.

## License

MIT License - see [LICENSE](./LICENSE) for details.

## Acknowledgments

Built with [Tauri](https://tauri.app/), [React](https://react.dev/), and [Rust](https://www.rust-lang.org/).

---

**Author**: AptS:1547 (Yuhan Bian / 卞雨涵)
**Repository**: [github.com/AptS-1547/dns-orchestrator](https://github.com/AptS-1547/dns-orchestrator)
