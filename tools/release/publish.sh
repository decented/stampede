#!/usr/bin/env bash
# Publish a Stampede release with nostr-release-kit's publisher and this repo's
# tools/release/release.json (channel, package, relays, certificate pin,
# release-key location). Usage:
#   tools/release/publish.sh <apk> --version X.Y.Z [--notes "..."] [--dry-run] [--allow-cert-change] [--allow-older]
#   tools/release/publish.sh --print-pubkey
# The versionCode is derived from --version (major * 1e6 + minor * 1e3 + patch),
# as tools/build-apk.sh stamps it into the APK.
# The kit's checkout: $KIT, default ../forgesworn/nostr-release-kit beside this repo
# (install its publisher's dependencies once: `npm ci` in its tools/), at the revision
# app/Cargo.toml pins. Env names are the kit's: RELEASE_KIT_RELAYS, RELEASE_KIT_APKSIGNER,
# RELEASE_KIT_AAPT2; the key file comes from $STAMPEDE_RELEASE_KEY_FILE (default
# ~/.stampede-release/release-key.hex).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KIT="${KIT:-$HERE/../../../forgesworn/nostr-release-kit}"
[ -f "$KIT/tools/publish-release.mjs" ] || { echo "ERROR: nostr-release-kit not found at $KIT (set KIT)" >&2; exit 1; }
KIT_REV="$(grep -o 'nostr-release-kit", rev = "[0-9a-f]*' "$HERE/../../app/Cargo.toml" | head -1 | grep -o '[0-9a-f]*$')"
KIT_HEAD="$(git -C "$KIT" rev-parse HEAD 2>/dev/null || true)"
[ -n "$KIT_REV" ] && [ "$KIT_HEAD" = "$KIT_REV" ] || { echo "ERROR: $KIT is at ${KIT_HEAD:-?}, app/Cargo.toml pins nostr-release-kit $KIT_REV (git -C \"$KIT\" checkout $KIT_REV)" >&2; exit 1; }
exec node "$KIT/tools/publish-release.mjs" --config "$HERE/release.json" "$@"
