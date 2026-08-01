#!/bin/bash
# AMOS Companion post-install hook
# Patches the .desktop file to add Categories= if left empty by Tauri bundler.
# Without Categories=, the app won't appear in Linux desktop app launchers.

set -e

# Find the .desktop file installed by this package
DESKTOP_FILE=""
for dir in /usr/share/applications /usr/local/share/applications; do
	if [ -f "$dir/amos-companion.desktop" ]; then
		DESKTOP_FILE="$dir/amos-companion.desktop"
		break
	fi
	# Also check for spaced filename (Tauri may generate it)
	for name in "amos-companion.desktop" "AMOS Companion.desktop"; do
		if [ -f "$dir/$name" ]; then
			DESKTOP_FILE="$dir/$name"
			break 2
		fi
	done
done

if [ -z "$DESKTOP_FILE" ] || [ ! -f "$DESKTOP_FILE" ]; then
	echo "[amos-companion postinst] .desktop file not found, skipping."
	exit 0
fi

# Check if Categories= is currently empty
CURRENT=$(grep '^Categories=' "$DESKTOP_FILE" 2>/dev/null | cut -d= -f2 | tr -d ' ')

if [ -z "$CURRENT" ]; then
	echo "[amos-companion postinst] Adding Categories to $DESKTOP_FILE"
	sed -i 's/^Categories=$/Categories=Utility;Network;RemoteAccess;DeveloperTools;/' "$DESKTOP_FILE"
else
	echo "[amos-companion postinst] Categories already set to '$CURRENT', skipping."
fi

exit 0
