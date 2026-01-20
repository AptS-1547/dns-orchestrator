# DNS Orchestrator TUI

A terminal user interface for DNS Orchestrator, providing keyboard-driven DNS management directly from your terminal.

## Features

- **Multi-Page Navigation** - Home, Domains, Accounts, Toolbox, Settings
- **Keyboard-Driven** - Full keyboard navigation with vim-style keybindings support
- **Cross-Platform** - Works on Windows, macOS, and Linux terminals
- **Elm Architecture** - Clean separation of Model, View, Update, and Message
- **Async Ready** - Built on Tokio for future async API operations

## Screenshots

```
┌──────────────────────────────────────────────────────────────┐
│ DNS Orchestrator TUI v0.1.0                    [Dark] [Ready]│
├─────────────┬────────────────────────────────────────────────┤
│             │                                                │
│  ● Home     │              Welcome to                        │
│    Domains  │         DNS Orchestrator TUI                   │
│    Accounts │                                                │
│    Toolbox  │    Manage your DNS records from terminal       │
│    Settings │                                                │
│             │                                                │
├─────────────┴────────────────────────────────────────────────┤
│ ↑↓ Navigate │ Enter Select │ ←→ Switch Panel │ q Quit       │
└──────────────────────────────────────────────────────────────┘
```

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/AptS-1547/dns-orchestrator.git
cd dns-orchestrator/dns-orchestrator-tui

# Build and run
cargo run --release
```

### Prerequisites

- Rust 1.70+
- A terminal with Unicode support

## Usage

### Quick Start

```bash
# Run the TUI
cargo run

# Or after building
./target/release/dns-orchestrator-tui
```

### Keyboard Shortcuts

#### Global

| Key | Action |
|-----|--------|
| `q` | Quit / Back |
| `Ctrl+C` | Force quit |
| `Alt+h` | Help |
| `Alt+r` | Refresh |
| `Esc` | Cancel / Back |
| `←` `→` | Switch panel focus |

#### Navigation Panel

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` | Select page |

#### Content Panel

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` | Confirm / Enter |
| `Tab` | Next item |
| `Shift+Tab` | Previous item |

#### Operations

| Key | Action |
|-----|--------|
| `Alt+a` | Add |
| `Alt+e` | Edit |
| `Alt+d` | Delete |
| `Alt+i` | Import |
| `Alt+x` | Export |

## Architecture

This project follows the **Elm Architecture (TEA)** pattern:

```
src/
├── main.rs           # Entry point
├── app.rs            # Main loop
├── model/            # Application state
├── message/          # Event definitions
├── update/           # State update logic
├── view/             # UI rendering
├── event/            # Input handling
├── backend/          # Business services
└── util/             # Utilities
```

### Data Flow

```
User Input → Event → Message → Update → Model → View → Terminal
```

## Configuration

Configuration is stored in the system config directory:

- **Linux**: `~/.config/dns-orchestrator-tui/`
- **macOS**: `~/Library/Application Support/dns-orchestrator-tui/`
- **Windows**: `%APPDATA%\dns-orchestrator-tui\`

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [dns-orchestrator-core](../dns-orchestrator-core) - Core DNS management library

## Development Status

This is an early development version. Current progress:

- [x] Basic framework setup
- [x] Main layout (left navigation + right content)
- [x] Keyboard event routing
- [x] Page navigation
- [ ] Accounts management UI
- [ ] Domains tree view
- [ ] DNS records CRUD
- [ ] Toolbox integration
- [ ] Settings page

## Contributing

See the main [DNS Orchestrator contributing guide](../README.md#contributing).

## License

MIT License - see [LICENSE](../LICENSE) for details.
