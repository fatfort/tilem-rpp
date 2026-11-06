#!/usr/bin/env python3
"""Write a reproducible zip: sorted entries, fixed timestamps, fixed modes.

Usage: make-zip.py <stage-dir> <out.zip> <unix-epoch>

Every entry gets the same timestamp (the commit time, UTC), mode 0755 for
executables and 0644 for everything else, and no extra fields, so the same
staged tree always gives the same bytes (with the same zlib).
"""
import os
import stat
import sys
import time
import zipfile


def main() -> None:
    stage, out, epoch = sys.argv[1], sys.argv[2], int(sys.argv[3])
    stamp = time.gmtime(max(epoch, 315532800))[:6]  # zip cannot go before 1980
    entries = []
    for root, dirs, files in os.walk(stage):
        for d in dirs:
            entries.append(os.path.relpath(os.path.join(root, d), stage) + "/")
        for f in files:
            entries.append(os.path.relpath(os.path.join(root, f), stage))
    entries.sort()
    with zipfile.ZipFile(out, "w") as z:
        for name in entries:
            info = zipfile.ZipInfo(name, date_time=stamp)
            info.create_system = 3
            path = os.path.join(stage, name)
            if name.endswith("/"):
                info.external_attr = (stat.S_IFDIR | 0o755) << 16 | 0x10
                z.writestr(info, b"")
                continue
            executable = os.stat(path).st_mode & stat.S_IXUSR
            info.external_attr = (stat.S_IFREG | (0o755 if executable else 0o644)) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            with open(path, "rb") as fh:
                z.writestr(info, fh.read(), compresslevel=9)


if __name__ == "__main__":
    main()
