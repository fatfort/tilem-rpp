# Changes relative to upstream

This file records what this project changed relative to the code it is derived
from (GPL-3.0 section 5a; LGPL-2.1 section 2). Licences and copyright holders
are in `NOTICE.txt`.

## Emulator core — `vendor/tilem/`: unchanged

The TilEm 2.0 emulator core (libtilemcore; Benjamin Moody, Solignac Julien) is
included **without modification**. The 81 files under `vendor/tilem/` are
byte-identical to `apps/tilem/emu/` in rM2-stuff (Timothy Werquin) at commit
`9126d7ff3093e5598342a7236f305e6331d9ed79`, which is where we copied them
from. Only the build files were not copied (`CMakeLists.txt`, `Makefile`,
`Makefile.in`, and a prebuilt `libtilemcore.a`). Every upstream copyright and
licence header is kept as it was.

The core is compiled the way upstream's `CMakeLists.txt` compiles it: every
`.c` file in `vendor/tilem/` and its model subdirectories, with
`PACKAGE_VERSION="2.0"`. `build.rs` adds `_GNU_SOURCE`, `-std=gnu99` and
warning-suppression flags only; no code is patched. Only the TI-84 Plus model
(`TILEM_CALC_TI84P`) is instantiated at run time.

## Front-end: new code (replaces rM2-stuff's C++ app)

rM2-stuff's `apps/tilem/main.cpp` drives the core on the reMarkable 2 through
its own C++ UI library. None of that UI code is used. Written for this
project, in Rust:

| File | What it does |
|---|---|
| `build.rs`, `wrapper.h` | Builds libtilemcore with `cc`, generates FFI bindings with `bindgen` |
| `src/bin/tilem/emu.rs` | Safe wrapper over `TilemCalc`: ROM load, state save/restore, LCD frame |
| `src/bin/tilem/main.rs` | ~100 Hz emulation loop, ROM-missing screen, save on SIGTERM/SIGINT |
| `src/bin/tilem/keymap.rs` | Keypad table — **derived from rM2-stuff** `apps/tilem/main.cpp` (11 rows of key labels and scancodes, translated to Rust), plus our own layout geometry |
| `src/bin/tilem/input.rs` | Touch → keypad routing, press feedback, minimum 50 ms hold so fast taps register |
| `src/bin/tilem/render.rs` | LCD upscaling, vector-drawn keypad, layout scaled from the screen width |
| `src/bin/tilem/rmpp_hal/` | QTFB display and input for rm-appload (our own HAL, shared with our other reMarkable apps) |
| `appload/` | AppLoad manifest and launcher icon |

## History

- 2026-04-19 — Initial port: Rust shell + QTFB backend around the
  vendored core; static aarch64-musl build via `cargo zigbuild`.
- 2026-04-19 — Key press feedback, deferred short releases, white
  LCD when the calculator display is off (the core leaves the off-state buffer
  uninitialised), launcher icon.
- 2026-04-21 — Runtime device detection: one binary runs on the
  Paper Pro (1620×2160) and the Paper Pro Move (954×1696); layout scales from
  the screen width.
- 2026-11-07 — Release 1.0.0: licence files and notices (`COPYING`,
  `LICENSES/`, `NOTICE.txt`), this file, `README.md`, a pinned containerised
  release build (`build.sh`, `docker/Dockerfile`, committed `Cargo.lock`) and
  packaging tests. No functional change to the program.
