# TilEm for reMarkable

A TI-84 Plus graphing-calculator emulator for the **reMarkable Paper Pro**
(11.8") and the **reMarkable Paper Pro Move** (7.3"), packaged as a standalone
AppLoad application. One binary runs on both tablets. It is built on the
TilEm 2.0 emulator core. **Bring your own TI-84 Plus ROM:** the emulator needs
a ROM image dumped from a TI-84 Plus you own, copied to
`/home/root/ti84plus.rom` on the tablet. No ROM is included, and we do not
provide ROMs or links to them.

The full calculator keypad is drawn on screen with shift and alpha labels;
tap keys with a finger. The emulated LCD is scaled up to fill the top of the
screen. The calculator's state is saved when you close the app and restored
the next time you open it.

## Requirements

- reMarkable Paper Pro or Paper Pro Move, in Developer Mode with SSH access.
- [xovi](https://github.com/asivery/xovi) and
  [rm-appload](https://github.com/asivery/rm-appload) installed on the tablet
  (both free, installed separately).
- A TI-84 Plus ROM image that you dumped from your own calculator (for
  example with TiLP's ROM dumper). Only the TI-84 Plus is emulated; ROMs for
  other models (TI-83 Plus, TI-84 Plus Silver Edition, TI-84 Plus CE, ...)
  are not supported.

## Install

1. Copy the `tilem/` folder from the release zip to
   `/home/root/xovi/exthome/appload/` on the tablet:
   ```
   scp -r tilem root@<tablet-ip>:/home/root/xovi/exthome/appload/
   ```
   You end up with:
   ```
   /home/root/xovi/exthome/appload/tilem/
     tilem                    # binary (aarch64 musl, static)
     icon.png                 # launcher icon
     external.manifest.json   # AppLoad manifest
   ```
2. Copy your ROM into place:
   ```
   scp ti84plus.rom root@<tablet-ip>:/home/root/ti84plus.rom
   ```
3. Open the **AppLoad** launcher on the tablet and tap **Tilem**.

If the ROM is missing or unreadable, the app shows a "ROM load failed" screen
with the path it expected. No xochitl restart is needed, and nothing in the
system UI is modified: this is an AppLoad app, a separate process launched by
rm-appload, so it carries no crash-loop or boot risk. To use a ROM at another
path, run the binary with the path as its first argument.

Saved state is written next to the ROM as `/home/root/ti84plus.rom.sav`.
Delete that file to start from a cold boot.

## Uninstall

```
rm -rf /home/root/xovi/exthome/appload/tilem
rm -f /home/root/ti84plus.rom.sav   # and your ROM, if you want it gone
```

## Building from source

The release binary is built by `./build.sh`, which runs the build in a pinned
container (`docker/Dockerfile`: Rust 1.94.1, cargo-zigbuild 0.22.1, zig 0.15.2,
libclang for bindgen) from `git archive HEAD`, then packages
`dist/tilem-<version>.zip`. It needs Docker and a clean checkout:

```
./build.sh
```

Without Docker, on a host with Rust, the `aarch64-unknown-linux-musl` target,
[cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild), zig and
libclang:

```
rustup target add aarch64-unknown-linux-musl
cargo install cargo-zigbuild
cargo zigbuild --release --locked --target aarch64-unknown-linux-musl
```

The static binary is at `target/aarch64-unknown-linux-musl/release/tilem`.
Copy it, `appload/external.manifest.json` and `appload/icon.png` into
`/home/root/xovi/exthome/appload/tilem/` on the tablet.

`tests/run.sh` checks the packaging (licence files present, no ROM in the zip,
no attribution trailers in the history) and, where a Rust toolchain is
installed, formatting.

## Licence

TilEm for reMarkable is free software, licensed under the **GNU General Public
License, version 3 or (at your option) any later version**: see `COPYING`.

    Copyright (C) 2026 Aayush Bajaj (reMarkable front-end)
    Copyright (C) 2001 Solignac Julien; (C) 2004-2012 Benjamin Moody (TilEm core)

The emulator core in `vendor/tilem/` is TilEm 2.0's libtilemcore, LGPL-2.1 or
later (`LICENSES/LGPL-2.1.txt`; one file, `memory.c`, is GPL-3.0 or later),
unchanged and with every upstream header kept. It was copied from
[rM2-stuff](https://github.com/timower/rM2-stuff) (GPL-3.0), from which the
keypad table in `src/bin/tilem/keymap.rs` is also derived; that is why the
whole program is GPL-3.0-or-later. The bundled Noto Sans font is under the SIL
Open Font License 1.1 (`LICENSES/OFL.txt`). Every component and its licence is
listed in `NOTICE.txt`; what we changed is in `CHANGES.md`.

Ready-to-install builds are sold at [bazaar.abaj.ai](https://bazaar.abaj.ai).
This repository, https://github.com/fatfort/tilem-move, is their complete
corresponding source; each release zip's `SOURCE.md` names the exact commit
and build recipe. You are paying for the port, packaging and testing; you are
free to build, modify and share it under the GPL.

## Trademarks and disclaimer

TI-84 Plus is a trademark of Texas Instruments Incorporated. This project is
not affiliated with or endorsed by Texas Instruments, and contains no Texas
Instruments software.

Compatible with reMarkable® Paper Pro and Paper Pro Move. Requires
reMarkable's official Developer Mode (enabling it factory-resets the device;
defects caused by software modifications are not covered by reMarkable's
warranty). Independent product — not affiliated with or endorsed by
reMarkable AS. Not affiliated with the TilEm or rM2-stuff authors.
