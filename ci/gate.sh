#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

SDK=rust

# >>> version-files (S3; must match versionFiles("rust") in ci/sdk-mirror/set-version.mjs)
VERSION_FILES=(
	Cargo.toml
)
# <<< version-files

fail() { echo "::error::$SDK: $1" >&2; exit 1; }
check() { local name=$1; shift; "$@" || fail "$name"; }

T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
export CARGO_TARGET_DIR=$T/target CARGO_TERM_COLOR=never

# Prints "<file> <field> <value>" per version field.
read_versions() {
	local f
	for f in "${VERSION_FILES[@]}"; do
		case $f in
			Cargo.toml) awk -v f="$f" '/^\[/ { s = $0 } s == "[package]" && /^version *= *"/ { gsub(/^version *= *"|".*$/, ""); print f, "[package].version", $0 }' "$f" ;;
			*) fail "version-files: no reader for $f" ;;
		esac
	done
}

check_versions() {
	local lines want
	lines=$(read_versions) || return 1
	[ "$(grep -c . <<<"$lines")" -eq 1 ] || { echo "  expected one [package] version, got: $lines" >&2; return 1; }
	want=${EXPECTED_VERSION:-$(head -1 <<<"$lines" | awk '{print $NF}')}
	echo "$lines" | awk -v want="$want" '$NF != want { print "  " $0 " != " want; bad=1 } END { exit bad }'
}

check_package_list() {
	local bad
	cargo package --list --allow-dirty >"$T/package.list" 2>"$T/package.err" || { cat "$T/package.err" >&2; return 1; }
	bad=$(grep -E '^(legacy|ci|\.github|rust-documentation)/|(^|/)LEGACY_[^/]*\.md$' "$T/package.list" || true)
	[ -z "$bad" ] || { echo "  unexpected package entries:" >&2; echo "$bad" >&2; return 1; }
	grep -qx Cargo.toml "$T/package.list" && grep -qx README.md "$T/package.list" && grep -q '^src/lib\.rs$' "$T/package.list" || { echo "  package missing Cargo.toml, README.md or src/lib.rs" >&2; return 1; }
}

echo "== $SDK: versions"; check versions check_versions
echo "== $SDK: cargo fmt"; check "cargo fmt" cargo fmt --check
echo "== $SDK: cargo build"; check "cargo build" cargo build --quiet --all-features
echo "== $SDK: cargo clippy"; check "cargo clippy" cargo clippy --quiet --all-features --all-targets -- -D warnings
echo "== $SDK: tests"; check tests cargo test --quiet --all-features
echo "== $SDK: examples"; check examples cargo build --quiet --examples
echo "== $SDK: package list"; check "package list" check_package_list
echo "== $SDK: publish dry-run"; check "publish dry-run" cargo publish --dry-run --allow-dirty --quiet
echo "== $SDK: cargo doc"; check "cargo doc" cargo doc --quiet --no-deps
echo "gate passed: $SDK"
