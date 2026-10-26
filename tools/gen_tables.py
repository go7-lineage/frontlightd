#!/usr/bin/env python3

"""
Regenerate src/tables_data.rs from decompiled stock DTBO

Usage: gen_tables.py <dtbo.dts> [src/tables_data.rs]
"""

import re
import sys

NAMES = [("ba", 0x03), ("bb", 0x04), ("ca", 0x05), ("cb", 0x06)]

HEADER = """\
//! LM3630A calibration tables extracted from stock DTBO 
//!
//! Regenerate with `tools/gen_tables.py <dtbo.dts>`

"""


def main(dts_path: str, out_path: str = "src/tables_data.rs") -> None:
    txt = open(dts_path).read()
    rows = []
    for name, reg in NAMES:
        m = re.search(r"table_" + name + r" = <([^>]*)>", txt)
        vals = [int(v, 16) for v in m.group(1).split()]
        assert len(vals) == 33 * 33, (name, len(vals))
        flipped = [v for r in reversed(range(33)) for v in vals[r * 33 : (r + 1) * 33]]
        body = "\n".join(
            "    ["
            + " ".join(f"0x{v:02x}," for v in flipped[i * 33 : (i + 1) * 33])
            + "],"
            for i in range(33)
        )
        rows.append(
            f"/// Codes for register {reg:#04x}, indexed by `[warmth][brightness]`, 0..=32 each\n"
            f"pub static {name.upper()}: [[u8; 33]; 33] = [\n{body}\n];\n"
        )
    with open(out_path, "w") as f:
        f.write(HEADER + "\n".join(rows))
    print(f"wrote {out_path}")


if __name__ == "__main__":
    main(*sys.argv[1:])
