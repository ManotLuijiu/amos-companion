#!/bin/sh
# AMOS Companion — Linux Installer
# Usage: curl -fsSL https://releases.amos.moo-vpn.online/install.sh | sh
#
# Options:
#   VERSION=x.y.z   Install a specific version (default: latest)
#   INSTALL_DIR=    Override install directory (default: ~/.local/bin)
#   SYSTEM_WIDE=1   Install to /usr/local/bin (requires sudo)

set -e

# ── Colours ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BOLD='\033[1m'; RESET='\033[0m'

info()    { printf "${GREEN}==>${RESET} ${BOLD}%s${RESET}\n" "$*"; }
warn()    { printf "${YELLOW}  ->%s${RESET}\n" "$*"; }
error()   { printf "${RED}ERROR:${RESET} %s\n" "$*"; }
ok()    { printf "${GREEN}  [OK] %s${RESET}\n" "$*"; }

# ── Defaults ───────────────────────────────────────────────────────────────────
RELEASE_BASE="${RELEASE_BASE:-https://releases.amos.moo-vpn.online/companion}"
VERSION="${VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
SYSTEM_WIDE="${SYSTEM_WIDE:-0}"
FORCE="${FORCE:-0}"

# ── Architecture ───────────────────────────────────────────────────────────────
detect_arch() {
    ARCH="$(uname -m)"
    case "$ARCH" in
        x86_64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)       echo "" ;;
    esac
}

ARCH="$(detect_arch)"
if [ -z "$ARCH" ]; then
    error "Unsupported architecture: $(uname -m)"
    error "Supported: x86_64, aarch64"
    exit 1
fi

# ── Detect installed version ───────────────────────────────────────────────────
get_installed_version() {
    if [ -x "${INSTALL_DIR}/amos-companion" ]; then
        "${INSTALL_DIR}/amos-companion" --version 2>/dev/null | \
            sed -E 's/.*v([0-9]+\.[0-9]+\.[0-9]+).*/\1/' || \
            echo "unknown"
    else
        echo "not-installed"
    fi
}

INSTALLED_VERSION="$(get_installed_version)"

# ── Resolve version ───────────────────────────────────────────────────────────
resolve_version() {
    if [ "$VERSION" = "latest" ]; then
        info "Checking for latest version..."
        MANIFEST_URL="${RELEASE_BASE}/manifest.json"
        LATEST_VERSION="$(curl -fsSL "$MANIFEST_URL" 2>/dev/null | \
            sed -E 's/.*"latest":"([^"]+)".*/\1/' || echo "")"
        if [ -z "$LATEST_VERSION" ]; then
            error "Failed to fetch latest version from manifest"
            exit 1
        fi
        echo "$LATEST_VERSION"
    else
        echo "$VERSION"
    fi
}

RESOLVED_VERSION="$(resolve_version)"

# ── URLs ──────────────────────────────────────────────────────────────────────
TARBALL="amos-companion-${ARCH}-${RESOLVED_VERSION}.tar.gz"
TARBALL_URL="${RELEASE_BASE}/${RESOLVED_VERSION}/${TARBALL}"
INSTALL_SCRIPT_URL="${RELEASE_BASE}/${RESOLVED_VERSION}/install.sh"

# ── Print summary ─────────────────────────────────────────────────────────────
echo ""
info "${BOLD}AMOS Companion${RESET} ${YELLOW}v${RESOLVED_VERSION}${RESET} for ${BOLD}${ARCH}${RESET}"
echo ""

if [ "$INSTALLED_VERSION" != "not-installed" ]; then
    if [ "$INSTALLED_VERSION" = "$RESOLVED_VERSION" ] && [ "$FORCE" = "0" ]; then
        info "Already on latest version (v${INSTALLED_VERSION})"
        exit 0
    else
        warn "Currently installed: v${INSTALLED_VERSION}"
        warn "Installing:        v${RESOLVED_VERSION}"
    fi
fi

# ── Check dependencies ─────────────────────────────────────────────────────────
check_deps() {
    MISSING=""
    for cmd in curl tar; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            MISSING="$MISSING $cmd"
        fi
    done
    if [ -n "$MISSING" ]; then
        error "Missing required commands:$MISSING"
        exit 1
    fi
}
check_deps

# ── Download ───────────────────────────────────────────────────────────────────
info "Downloading AMOS Companion..."
TARBALL_PATH="/tmp/${TARBALL}"

if ! curl -fSL "$TARBALL_URL" -o "$TARBALL_PATH" --progress-bar; then
    error "Download failed: $TARBALL_URL"
    error "Version $RESOLVED_VERSION may not be available for $ARCH"
    exit 1
fi

ok "Downloaded $TARBALL ($(du -h "$TARBALL_PATH" | cut -f1))"

# ── Verify tarball ─────────────────────────────────────────────────────────────
info "Verifying archive..."
if ! tar -tzf "$TARBALL_PATH" >/dev/null 2>&1; then
    error "Invalid or corrupted archive"
    rm -f "$TARBALL_PATH"
    exit 1
fi
ok "Archive verified"

# ── Install binary ─────────────────────────────────────────────────────────────
info "Installing to ${INSTALL_DIR}..."

if [ "$SYSTEM_WIDE" = "1" ]; then
    if [ "$(id -u)" -ne 0 ]; then
        error "SYSTEM_WIDE=1 requires root (sudo)"
        rm -f "$TARBALL_PATH"
        exit 1
    fi
    EXTRACT_DIR="/tmp/amos-companion-install"
    rm -rf "$EXTRACT_DIR"
    mkdir -p "$EXTRACT_DIR"
    tar -xzf "$TARBALL_PATH" -C "$EXTRACT_DIR"
    cp "$EXTRACT_DIR/amos-companion" "${INSTALL_DIR}/amos-companion"
    chmod 755 "${INSTALL_DIR}/amos-companion"
    rm -rf "$EXTRACT_DIR"
else
    mkdir -p "$INSTALL_DIR"
    tar -xzf "$TARBALL_PATH" -C "$INSTALL_DIR" --strip-components=1
fi

rm -f "$TARBALL_PATH"
ok "Binary installed to ${INSTALL_DIR}/amos-companion"

# ── Desktop integration ────────────────────────────────────────────────────────
install_desktop() {
    DESKTOP_DIR="${HOME}/.local/share/applications"
    ICON_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"
    AUTOSTART_DIR="${HOME}/.config/autostart"

    mkdir -p "$DESKTOP_DIR" "$ICON_DIR" "$AUTOSTART_DIR"

    # .desktop file
    cat > "${DESKTOP_DIR}/amos-companion.desktop" << 'DESKTOP_EOF'
[Desktop Entry]
Name=AMOS Companion
Comment=Android device mirror & control for AMOS
Exec=amos-companion
Icon=amos-companion
Terminal=false
Type=Application
Categories=Utility;Network;
Keywords=android;mirror;scrcpy;adb;device;
MimeType=x-scheme-handler/amos;
StartupNotify=true
X-GNOME-Autostart-enabled=true
DESKTOP_EOF
    chmod 644 "${DESKTOP_DIR}/amos-companion.desktop"
    ok "Desktop entry created"

    # Auto-start
    if [ -f "${AUTOSTART_DIR}/amos-companion.desktop" ]; then
        warn "Auto-start already enabled"
    else
        cat > "${AUTOSTART_DIR}/amos-companion.desktop" << 'AUTOSTART_EOF'
[Desktop Entry]
Name=AMOS Companion
Exec=amos-companion
Hidden=false
X-GNOME-Autostart-enabled=true
AUTOSTART_EOF
        chmod 644 "${AUTOSTART_DIR}/amos-companion.desktop"
        ok "Auto-start enabled"
    fi

    # Update desktop database
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    fi
}

info "Setting up desktop integration..."
install_desktop

# ── Done ─────────────────────────────────────────────────────────────────────
echo ""
info "${BOLD}AMOS Companion v${RESOLVED_VERSION} installed!${RESET}"
echo ""
if command -v amos-companion >/dev/null 2>&1; then
    CURRENT_PATH="$(command -v amos-companion)"
    if [ "$CURRENT_PATH" != "${INSTALL_DIR}/amos-companion" ]; then
        warn "amos-companion found at $CURRENT_PATH (not in PATH)"
        warn "Add ${INSTALL_DIR} to your PATH to use 'amos-companion' from anywhere"
    fi
else
    warn "${INSTALL_DIR} is not in your PATH"
    warn "Add this to your ~/.bashrc or ~/.zshrc:"
    warn "  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi
echo ""
info "Run: ${BOLD}amos-companion${RESET}"
echo ""
