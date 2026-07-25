# AMOS Companion App

One-stop CLI app to connect Android devices to AMOS SaaS.

## Features

- 🔐 **Secure auth** — API key exchange for short-lived bearer tokens
- 📱 **USB auto-discovery** — Detects connected Android devices via ADB
- 🔄 **Auto-reconnect** — Handles network interruptions gracefully
- 🔄 **Auto-update** — Stays current with the latest AMOS agent
- 🌐 **Cross-platform** — Windows, macOS, Linux

## Quick Install

### Linux

```bash
# Debian/Ubuntu (.deb)
curl -fSL https://install.amos.moo-vpn.online/companion/amos-companion_0.1.0_amd64.deb -o /tmp/amos-companion.deb && sudo apt-get install -y /tmp/amos-companion.deb

# AppImage (portable, no install required)
curl -fSL https://install.amos.moo-vpn.online/companion/amos-companion_0.1.0_amd64.AppImage -o ~/amos-connect && chmod +x ~/amos-connect

# or via install script
curl -fsSL https://install.amos.moo-vpn.online/install.sh | bash
```

### macOS

```bash
curl -fSL https://install.amos.moo-vpn.online/companion/AMOS\ Companion_0.1.0_aarch64.dmg -o /tmp/amos-companion.dmg
open /tmp/amos-companion.dmg
# Drag AMOS Companion.app to Applications
```

### Windows

```powershell
irm https://install.amos.moo-vpn.online/companion/AMOS\ Companion_0.1.0_x64-setup.exe -o $env:TEMP\amos-companion.exe; & $env:TEMP\amos-companion.exe
```

### From npm (requires Node.js)

```bash
npm install -g amos-companion
amos-connect --configure
```

## Usage

```bash
amos-connect --configure          # First-time setup
amos-connect                     # Start agent
amos-connect --status           # Check status
amos-connect --uninstall         # Remove from system
```

## First-Time Setup

1. Download the AMOS Companion App from <https://app.amos.moo-vpn.online/settings/devices>
2. Run `amos-connect --configure`
3. Enter your API Key from the dashboard
4. Connect your Android tablet via USB with USB debugging enabled
5. The device will appear in your AMOS dashboard

## Requirements

- **Node.js** >= 18
- **Python** >= 3.11 (for the device-agent)
- **ADB** (Android SDK Platform Tools)
- **Git**

## How It Works

```
┌──────────────┐    HTTPS Bearer     ┌──────────────┐
│  AMOS        │◄──────────────────►│   AMOS       │
│  Companion   │                     │   API        │
│  CLI         │                     │   Server     │
└──────┬───────┘                     └──────────────┘
       │ USB
       ▼
┌──────────────┐
│   Android    │
│   Tablet     │
└──────────────┘
```

## Development

```bash
cd companion
npm install
npm run build
node dist/cli.js --dry-run
```

## License

Proprietary — AMOS
