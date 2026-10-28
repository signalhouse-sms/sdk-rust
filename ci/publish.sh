#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

SDK=rust
fail() { echo "::error::$SDK: $1" >&2; exit 1; }
run() { if [ "${PUBLISH_DRY_RUN:-}" = 1 ]; then printf '+ (dry run) would run:'; printf ' %q' "$@"; echo; else "$@"; fi; }

# Exit 0 = published, 1 = absent; anything else fails. crates.io rejects requests without a User-Agent.
published() {
	local code
	code=$(curl -sS -o /dev/null -w '%{http_code}' --retry 3 -A "signalhouse-sms-release (https://github.com/signalhouse-sms/sdk-rust)" "https://crates.io/api/v1/crates/$1/$2") || fail "registry check $1 $2"
	case $code in
		200) return 0 ;;
		404) return 1 ;;
		*) fail "registry check $1 $2: HTTP $code" ;;
	esac
}

V=$(awk '/^\[/ { s = $0 } s == "[package]" && /^version *= *"/ { gsub(/^version *= *"|".*$/, ""); print; exit }' Cargo.toml)
[ -n "$V" ] || fail "read [package] version"
if published signalhouse-sms "$V"; then echo "signalhouse-sms $V exists, skipping"; exit 0; fi

T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
export CARGO_TARGET_DIR=$T/target
run cargo publish --dry-run || fail "cargo publish --dry-run"
run cargo publish || fail "cargo publish"
