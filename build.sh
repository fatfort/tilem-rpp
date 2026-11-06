#!/bin/sh
# Build the TilEm for reMarkable release zip: dist/tilem-<version>.zip
#
# Usage: ./build.sh
#
# Everything in the zip comes from `git archive HEAD` (uncommitted edits never
# enter it): the binary is built from that archive inside the pinned container
# in docker/Dockerfile, and the docs/licence files are taken from the same
# archive. The zip is reproducible: sorted entries, the commit time as every
# timestamp, fixed modes (scripts/make-zip.py).
#
# Env: REPO_URL   public source repository named in SOURCE.md
#      IMAGE      docker image tag for the toolchain
set -eu
cd "$(dirname "$0")"

REPO_URL=${REPO_URL:-https://github.com/fatfort/tilem-move}
IMAGE=${IMAGE:-tilem-rm-build:1.94.1}
TARGET=aarch64-unknown-linux-musl

COMMIT=$(git rev-parse HEAD)
SHORT=$(git rev-parse --short=7 HEAD)
EPOCH=$(git log -1 --format=%ct HEAD)
VERSION=$(git show HEAD:Cargo.toml | sed -n 's/^version = "\(.*\)"$/\1/p' | sed -n 1p)
[ -n "$VERSION" ] || { echo "build.sh: no version in Cargo.toml" >&2; exit 1; }
if ! git diff --quiet HEAD --; then
    echo "build.sh: note: uncommitted changes are NOT in the build (it uses HEAD $SHORT)" >&2
fi

DIST=$PWD/dist
WORK=$DIST/work-$SHORT
ZIP=$DIST/tilem-$VERSION.zip
rm -rf "$WORK"
mkdir -p "$WORK/src" "$WORK/stage/tilem" "$WORK/stage/LICENSES"
git archive --format=tar HEAD > "$WORK/source.tar"
tar -xf "$WORK/source.tar" -C "$WORK/src"

# ---- binary (pinned container) --------------------------------------------
docker build -q -t "$IMAGE" "$WORK/src/docker" >/dev/null
docker run --rm -v "$WORK:/out" "$IMAGE" sh -euc '
  tar -xf /out/source.tar -C /build
  { rustc -vV; cargo -V; echo "cargo-zigbuild $(cargo-zigbuild --version | sed "s/^cargo-zigbuild //")";
    echo "zig $(zig version)"; dpkg-query -W -f "\${Package} \${Version}\n" libclang-dev libclang1-14; } > /out/toolchain.txt
  CARGO_PROFILE_RELEASE_STRIP=symbols cargo zigbuild --release --locked --target '"$TARGET"' > /out/build.log 2>&1 \
    || { tail -40 /out/build.log; exit 1; }
  cp target/'"$TARGET"'/release/tilem /out/tilem'
file "$WORK/tilem" | grep -q 'ARM aarch64.*statically linked' \
    || { echo "build.sh: binary is not a static aarch64 executable: $(file "$WORK/tilem")" >&2; exit 1; }
BIN_SHA=$(shasum -a 256 "$WORK/tilem" | cut -d' ' -f1)

# ---- stage ----------------------------------------------------------------
S=$WORK/stage
cp "$WORK/tilem" "$S/tilem/tilem"
chmod 0755 "$S/tilem/tilem"
cp "$WORK/src/appload/icon.png" "$WORK/src/appload/external.manifest.json" "$S/tilem/"
cp "$WORK/src/COPYING" "$WORK/src/NOTICE.txt" "$WORK/src/README.md" "$WORK/src/CHANGES.md" "$S/"
cp "$WORK/src/LICENSES/"* "$S/LICENSES/"

RUSTC=$(sed -n 's/^rustc \(.*\)$/\1/p' "$WORK/toolchain.txt" | sed -n 1p)
{
cat <<MD
# TilEm for reMarkable $VERSION — source and build recipe

Source offer: the complete corresponding source for the binary in this zip is
available, free of charge, at $REPO_URL.

| | |
|---|---|
| Repository | $REPO_URL |
| Commit | \`$COMMIT\` |
| Tag | \`v$VERSION\` |
| Licence | GPL-3.0-or-later (\`COPYING\`); core LGPL-2.1-or-later (\`LICENSES/\`) |
| Target | reMarkable Paper Pro and Paper Pro Move (one binary; device detected at run time) |
| Binary | \`tilem/tilem\`, $TARGET, static, stripped |
| Binary sha256 | \`$BIN_SHA\` |

No calculator ROM is included. Supply a TI-84 Plus ROM dumped from your own
calculator at \`/home/root/ti84plus.rom\` (see \`README.md\`).

The sha256 of this zip itself cannot be recorded inside it; it is published
next to the download (\`tilem-$VERSION.zip.sha256\`).

## Toolchain

- rustc $RUSTC, image \`rust:1.94.1-slim-bookworm\` pinned by digest (linux/arm64)
- cargo-zigbuild 0.22.1, zig 0.15.2 (C compiler and linker)
- libclang 14 from Debian bookworm (bindgen, build time only)
- Dependencies exactly as pinned in the repository's \`Cargo.lock\` (\`--locked\`)

Versions reported inside the container for this build:

\`\`\`
$(cat "$WORK/toolchain.txt")
\`\`\`

## Build

From a clone of the repository at the commit above, with Docker:

\`\`\`sh
git checkout $COMMIT
./build.sh          # writes dist/tilem-$VERSION.zip and dist/tilem-$VERSION.zip.sha256
\`\`\`

\`build.sh\` runs \`git archive HEAD\`, builds the binary from that archive in
the container below, and packages the zip reproducibly. The binary step on
its own:

\`\`\`sh
git archive --format=tar $COMMIT > source.tar
docker build -t $IMAGE docker/
docker run --rm -v "\$PWD:/out" $IMAGE sh -euc '
  tar -xf /out/source.tar -C /build
  CARGO_PROFILE_RELEASE_STRIP=symbols \\
    cargo zigbuild --release --locked --target $TARGET
  cp target/$TARGET/release/tilem /out/tilem'
sha256sum tilem
\`\`\`

\`docker/Dockerfile\`:

\`\`\`dockerfile
$(cat "$WORK/src/docker/Dockerfile")
\`\`\`

Without Docker, \`cargo zigbuild --release --locked --target $TARGET\` on a
host with the tools above produces a functionally identical binary;
byte-identity needs the same toolchain versions, the \`/build\` source path and
symbol stripping.
MD
} > "$S/SOURCE.md"

# ---- zip --------------------------------------------------------------------
rm -f "$ZIP" "$ZIP.sha256"
python3 -I "$WORK/src/scripts/make-zip.py" "$S" "$ZIP" "$EPOCH"
( cd "$DIST" && shasum -a 256 "$(basename "$ZIP")" > "$(basename "$ZIP").sha256" )
cp "$WORK/tilem" "$DIST/tilem-$VERSION-$SHORT.bin"

echo "commit   $COMMIT"
echo "binary   $BIN_SHA"
echo "zip      $(cut -d' ' -f1 "$ZIP.sha256")  $ZIP"
