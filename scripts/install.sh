#!/bin/sh
# AMOS Companion — Linux Installer
# Usage: curl -fsSL https://pub-a612420d2b9c409b9017805eabd46301.r2.dev/companion/latest/install.sh | sh
#
# Options:
#   VERSION=x.y.z   Install a specific version (default: latest)
#   INSTALL_DIR=    Override install directory (default: ~/.local/bin)
#   SYSTEM_WIDE=1   Install to /usr/local/bin (requires sudo)
#
# ─── Ubuntu/Debian .deb users ─────────────────────────────────────────────────
# If you installed via .deb and need to update:
#   sudo apt install ./amos-companion_<arch>_<version>.deb
#
# If App Center or double-click doesn't work, use terminal:
#   sudo dpkg -i ./amos-companion_<arch>_<version>.deb
#   sudo apt -f install  # Fix any dependency issues
# ────────────────────────────────────────────────────────────────────────────

set -e

# ── Colours ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
RESET='\033[0m'

info() { printf "${GREEN}==>${RESET} ${BOLD}%s${RESET}\n" "$*"; }
warn() { printf "${YELLOW}  ->%s${RESET}\n" "$*"; }
error() { printf "${RED}ERROR:${RESET} %s\n" "$*"; }
ok() { printf "${GREEN}  [OK] %s${RESET}\n" "$*"; }

# ── Defaults ───────────────────────────────────────────────────────────────────
RELEASE_BASE="${RELEASE_BASE:-https://pub-a612420d2b9c409b9017805eabd46301.r2.dev/companion}"
VERSION="${VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
SYSTEM_WIDE="${SYSTEM_WIDE:-0}"
FORCE="${FORCE:-0}"

# ── Architecture ───────────────────────────────────────────────────────────────
detect_arch() {
	ARCH="$(uname -m)"
	case "$ARCH" in
	x86_64) echo "amd64" ;;
	aarch64 | arm64) echo "aarch64" ;;
	*) echo "" ;;
	esac
}

ARCH="$(detect_arch)"
if [ -z "$ARCH" ]; then
	error "Unsupported architecture: $(uname -m)"
	error "Supported: x86_64, aarch64"
	exit 1
fi

# ── Remove existing .deb package if installed ───────────────────────────────────
remove_deb_if_exists() {
	if command -v dpkg-query >/dev/null 2>&1; then
		if dpkg-query -W -f='${Status}' amos-companion 2>/dev/null | grep -q "installed"; then
			warn "Found existing .deb installation (amos-companion)"
			warn "Removing .deb package to avoid conflicts..."
			if command -v sudo >/dev/null 2>&1; then
				sudo dpkg -r amos-companion 2>/dev/null || true
			else
				dpkg -r amos-companion 2>/dev/null || true
			fi
			ok ".deb package removed"
		fi
	fi
}

# ── Detect installed version ────────────────────────────────────────────────────
get_installed_version() {
	if [ -x "${INSTALL_DIR}/amos-companion" ]; then
		"${INSTALL_DIR}/amos-companion" --version 2>/dev/null |
			sed -E 's/.*v([0-9]+\.[0-9]+\.[0-9]+).*/\1/' ||
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
		LATEST_VERSION="$(curl -fsSL "$MANIFEST_URL" 2>/dev/null |
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
_INSTALL_SCRIPT_URL="${RELEASE_BASE}/${RESOLVED_VERSION}/install.sh"

# ── Print summary ─────────────────────────────────────────────────────────────
echo ""
info "${BOLD}AMOS Companion${RESET} ${YELLOW}v${RESOLVED_VERSION}${RESET} for ${BOLD}${ARCH}${RESET}"
echo ""

# Remove any existing .deb package to avoid conflicts
remove_deb_if_exists

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
	tar -xzf "$TARBALL_PATH" -C "$INSTALL_DIR"
fi

rm -f "$TARBALL_PATH"
ok "Binary installed to ${INSTALL_DIR}/amos-companion"

# ── Desktop integration ────────────────────────────────────────────────────────
install_desktop() {
	DESKTOP_DIR="${HOME}/.local/share/applications"
	ICON_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"
	AUTOSTART_DIR="${HOME}/.config/autostart"

	mkdir -p "$DESKTOP_DIR" "$ICON_DIR" "$AUTOSTART_DIR"

	# .desktop file (use absolute icon path)
	cat >"${DESKTOP_DIR}/amos-companion.desktop" <<DESKTOP_EOF
[Desktop Entry]
Name=AMOS Companion
Comment=Android device mirror & control for AMOS
Exec=${INSTALL_DIR}/amos-companion
Icon=${HOME}/.local/share/icons/hicolor/256x256/apps/amos-companion.png
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

	# Download icon
	ICON_URL="${RELEASE_BASE}/${RESOLVED_VERSION}/icon.png"
	if curl -fsSL "$ICON_URL" -o "${ICON_DIR}/amos-companion.png"; then
		ok "Icon downloaded"
	else
		warn "Icon not found (will use default)"
	fi

	# Auto-start
	if [ -f "${AUTOSTART_DIR}/amos-companion.desktop" ]; then
		warn "Auto-start already enabled"
	else
		cat >"${AUTOSTART_DIR}/amos-companion.desktop" <<'AUTOSTART_EOF'
[Desktop Entry]
Name=AMOS Companion
Exec=${INSTALL_DIR}/amos-companion
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

# ── Uninstall ────────────────────────────────────────────────────────────────
uninstall() {
	info "Uninstalling AMOS Companion..."

	# Stop the app if running
	if pgrep -x amos-companion >/dev/null 2>&1; then
		info "Stopping AMOS Companion..."
		pkill amos-companion 2>/dev/null || true
		sleep 1
	fi

	# Remove binary
	if [ -f "${INSTALL_DIR}/amos-companion" ]; then
		rm -f "${INSTALL_DIR}/amos-companion"
		ok "Binary removed"
	fi

	# Remove scrcpy server
	if [ -f "${INSTALL_DIR}/scrcpy-server.jar" ]; then
		rm -f "${INSTALL_DIR}/scrcpy-server.jar"
		ok "scrcpy-server removed"
	fi

	# Remove desktop entry
	DESKTOP_DIR="${HOME}/.local/share/applications"
	if [ -f "${DESKTOP_DIR}/amos-companion.desktop" ]; then
		rm -f "${DESKTOP_DIR}/amos-companion.desktop"
		ok "Desktop entry removed"
	fi

	# Remove autostart entry
	AUTOSTART_DIR="${HOME}/.config/autostart"
	if [ -f "${AUTOSTART_DIR}/amos-companion.desktop" ]; then
		rm -f "${AUTOSTART_DIR}/amos-companion.desktop"
		ok "Auto-start entry removed"
	fi

	# Remove icon
	ICON_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"
	if [ -f "${ICON_DIR}/amos-companion.png" ]; then
		rm -f "${ICON_DIR}/amos-companion.png"
		ok "Icon removed"
	fi

	# Remove config (ask first)
	CONFIG_DIR="${HOME}/.config/amos-companion"
	if [ -f "${CONFIG_DIR}/config.toml" ]; then
		echo ""
		printf "%sRemove config and data (%s)? [y/N]: %s" "$YELLOW" "$CONFIG_DIR" "$RESET"
		read -r response
		case "$response" in
		[yY])
			rm -rf "${CONFIG_DIR}"
			ok "Config removed"
			;;
		*)
			warn "Config kept at ${CONFIG_DIR}"
			;;
		esac
	fi

	# Remove logs
	LOG_DIR="${HOME}/.local/share/amos-companion"
	if [ -d "${LOG_DIR}" ]; then
		rm -rf "${LOG_DIR}"
		ok "Logs removed"
	fi

	echo ""
	info "AMOS Companion uninstalled!"
	info "Run '${BOLD}amos-companion${RESET}' to reinstall."
}

# Check for uninstall flag
case "$1" in
--uninstall | -u)
	INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
	uninstall
	exit 0
	;;
esac
