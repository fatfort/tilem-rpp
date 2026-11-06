#!/bin/sh
# Release checks for TilEm for reMarkable.
#
# Usage: tests/run.sh [path/to/tilem-<version>.zip]
#   default zip: dist/tilem-<version from Cargo.toml>.zip (skipped if absent)
#
# Checks: shell syntax; licence/notice files in the tree; Cargo licence field;
# no Co-Authored-By / Signed-off-by / Generated-by trailers in the history of
# HEAD; one consistent source URL; zip contents (required files present, no
# ROM/state/OS image, binary sha matches SOURCE.md); cargo fmt/check when a
# Rust toolchain is installed.
set -u
cd "$(dirname "$0")/.."
FAIL=0
pass() { echo "ok    $1"; }
fail() { echo "FAIL  $1"; FAIL=1; }
skip() { echo "skip  $1"; }

# --- shell syntax ----------------------------------------------------------
for f in build.sh tests/run.sh; do
    if sh -n "$f" 2>/dev/null; then pass "sh -n $f"; else fail "sh -n $f"; fi
done
if python3 -I -m py_compile scripts/make-zip.py 2>/dev/null; then pass "py_compile make-zip.py"
else fail "py_compile make-zip.py"; fi
rm -rf scripts/__pycache__

# --- licence files in the tree ----------------------------------------------
for f in COPYING LICENSES/LGPL-2.1.txt LICENSES/OFL.txt NOTICE.txt README.md CHANGES.md Cargo.lock; do
    if [ -s "$f" ]; then pass "present $f"; else fail "missing $f"; fi
done
grep -q 'GNU GENERAL PUBLIC LICENSE' COPYING && grep -q 'Version 3, 29 June 2007' COPYING \
    && pass "COPYING is GPL-3.0" || fail "COPYING is not the GPL-3.0 text"
grep -q 'Version 2.1, February 1999' LICENSES/LGPL-2.1.txt && pass "LGPL-2.1 text" || fail "LGPL-2.1 text"
grep -q 'SIL OPEN FONT LICENSE Version 1.1' LICENSES/OFL.txt && pass "OFL text" || fail "OFL text"
grep -q '^license = "GPL-3.0-or-later"$' Cargo.toml && pass "Cargo.toml licence" || fail "Cargo.toml licence"
for w in "Benjamin Moody" "Solignac Julien" "rM2-stuff" "Noto" "Texas Instruments"; do
    grep -q "$w" NOTICE.txt && pass "NOTICE names $w" || fail "NOTICE missing $w"
done
grep -q '/home/root/ti84plus.rom' README.md && grep -q 'we do not' README.md \
    && pass "README: BYO ROM + no ROMs/links" || fail "README ROM statement"
sed -n 1,12p README.md | grep -q 'Bring your own TI-84 Plus ROM' \
    && pass "README: BYO ROM in first paragraph" || fail "README: BYO ROM not up front"
n=$(grep -L 'Copyright' $(git ls-files 'vendor/*.c' 'vendor/*.h') | grep -v 'vendor/tilem/tilemint.h' | wc -l | tr -d ' ')
[ "$n" = 0 ] && pass "vendor headers kept" || fail "$n vendor files lost their copyright header"

# --- history trailers ---------------------------------------------------------
if git log --format=%B HEAD | grep -qiE '^(co-authored-by|signed-off-by|generated[- ]by):'; then
    fail "attribution trailers in history: $(git log --format='%h %B' HEAD | grep -ciE '^(co-authored-by|signed-off-by|generated[- ]by):') line(s)"
    git log --format='%h %s' --grep='Co-Authored-By' -i HEAD | sed 's/^/        /'
else
    pass "no attribution trailers in history"
fi

# --- one source URL -----------------------------------------------------------
urls=$(grep -ohE 'https://github.com/fatfort/[A-Za-z0-9_.-]+' README.md build.sh Cargo.toml | sort -u)
[ "$(echo "$urls" | wc -l | tr -d ' ')" = 1 ] && pass "single source URL: $urls" \
    || fail "inconsistent source URLs: $(echo $urls)"

# --- zip ------------------------------------------------------------------------
VERSION=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | sed -n 1p)
ZIP=${1:-dist/tilem-$VERSION.zip}
if [ -f "$ZIP" ]; then
    list=$(unzip -Z1 "$ZIP")
    for f in tilem/tilem tilem/external.manifest.json tilem/icon.png COPYING \
             LICENSES/LGPL-2.1.txt LICENSES/OFL.txt NOTICE.txt README.md CHANGES.md SOURCE.md; do
        echo "$list" | grep -qx "$f" && pass "zip has $f" || fail "zip lacks $f"
    done
    bad=$(echo "$list" | grep -iE '\.(rom|sav|8xu|8xk|bin|img)$|ti8[34]' || true)
    [ -z "$bad" ] && pass "zip has no ROM/state/OS image" || fail "zip carries: $bad"
    big=$(unzip -Zl "$ZIP" | awk '/^-/ && $4 >= 131072 && $NF != "tilem/tilem" {print $NF}')
    [ -z "$big" ] && pass "no unexpected large members" || fail "unexpected large members: $big"
    T=$(mktemp -d)
    unzip -q "$ZIP" -d "$T"
    sha=$(shasum -a 256 "$T/tilem/tilem" | cut -d' ' -f1)
    grep -q "$sha" "$T/SOURCE.md" && pass "SOURCE.md binary sha matches" || fail "SOURCE.md binary sha mismatch"
    grep -q "$(git rev-parse HEAD)" "$T/SOURCE.md" && pass "SOURCE.md commit = HEAD" \
        || echo "warn  SOURCE.md commit is not HEAD (zip built from another commit)"
    if command -v file >/dev/null; then
        file "$T/tilem/tilem" | grep -q 'ARM aarch64.*statically linked' && pass "binary static aarch64" || fail "binary type"
    fi
    [ -x "$T/tilem/tilem" ] && pass "binary executable bit" || fail "binary not executable"
    if grep -aqiE 'co-authored|anthropic' "$T/tilem/tilem"; then fail "binary contains attribution strings"
    else pass "binary free of attribution strings"; fi
    for f in "$T"/*.md "$T"/NOTICE.txt; do
        if grep -qiE 'co-authored-by|generated (by|with) claude|anthropic' "$f"; then fail "signature text in $(basename "$f")"; fi
    done
    if grep -q 'aayushbajaj7@gmail' -r "$T"; then fail "personal email in zip"; else pass "no personal email in zip"; fi
    if [ -f "$ZIP.sha256" ]; then
        (cd "$(dirname "$ZIP")" && shasum -a 256 -c "$(basename "$ZIP").sha256" >/dev/null 2>&1) \
            && pass "zip sha256 file matches" || fail "zip sha256 file mismatch"
    fi
    rm -rf "$T"
else
    skip "zip checks ($ZIP not built)"
fi

# --- rust toolchain (optional) -------------------------------------------------
if command -v cargo >/dev/null 2>&1; then
    cargo fmt --check >/dev/null 2>&1 && pass "cargo fmt --check" || fail "cargo fmt --check"
    cargo check --locked --target aarch64-unknown-linux-musl >/dev/null 2>&1 && pass "cargo check" || fail "cargo check"
elif [ "${TILEM_TEST_DOCKER:-0}" = 1 ] && docker image inspect tilem-rm-build:1.94.1 >/dev/null 2>&1; then
    # Report-only: the 1.0.0 sources predate rustfmt use (formatting is not gated).
    if docker run --rm -v "$PWD:/src:ro" tilem-rm-build:1.94.1 sh -c \
        'rustup component add rustfmt >/dev/null 2>&1 && cd /src && cargo fmt --check' >/dev/null 2>&1; then
        pass "cargo fmt --check (container)"
    else
        echo "warn  cargo fmt --check reports diffs (container; not gated)"
    fi
    skip "cargo check (build.sh compiles in the container)"
else
    skip "cargo fmt/check (no cargo on PATH; build.sh compiles in the container; TILEM_TEST_DOCKER=1 for fmt)"
fi

[ "$FAIL" = 0 ] && echo "ALL PASS" || echo "FAILURES"
exit "$FAIL"
