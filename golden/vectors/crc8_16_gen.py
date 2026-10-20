#!/usr/bin/env python3
"""Writes golden/vectors/crc8_16.txt (CRC-8/AUTOSAR, CRC-16/IBM-3740 and transmission CRC
test vectors, see golden/README.md).

The expected values come from the plain bit-wise implementations below (no tables), which
are independent of the bitis runtimes. The transmission frames include all test vectors of
carachangprotocol/FRAMING.md section 5; the script asserts that the computed headers and
trailers equal the values of that table (copied into FRAMING_TABLE below).

    python3 golden/vectors/crc8_16_gen.py          rewrite crc8_16.txt and its Dart copy
    python3 golden/vectors/crc8_16_gen.py --check  exit 1 if one of them is out of date

The Dart copy golden/dart/test/crc8_16_vectors.g.dart embeds crc8_16.txt as a string
constant for the web platforms (no file I/O), like crc32_gen.py does for crc32.txt.
"""
import pathlib
import sys


def crc8_autosar(data):
    """CRC-8/AUTOSAR: poly 0x2f, init 0xff, xorout 0xff, not reflected."""
    c = 0xFF
    for b in data:
        c ^= b
        for _ in range(8):
            c = ((c << 1) ^ 0x2F) & 0xFF if c & 0x80 else (c << 1) & 0xFF
    return c ^ 0xFF


def crc16_ibm3740(data):
    """CRC-16/IBM-3740 (CCITT-FALSE): poly 0x1021, init 0xffff, xorout 0, not reflected."""
    c = 0xFFFF
    for b in data:
        c ^= b << 8
        for _ in range(8):
            c = ((c << 1) ^ 0x1021) & 0xFFFF if c & 0x8000 else (c << 1) & 0xFFFF
    return c


def transmission_trailer(frame):
    """FRAMING.md section 2: n <= 14 -> CRC-8 (1 byte), n >= 15 -> CRC-16 little-endian."""
    if len(frame) <= 14:
        return bytes([crc8_autosar(frame)])
    return crc16_ibm3740(frame).to_bytes(2, "little")


def xtx_header(is_encrypted, size):
    """bitis encoding of XTX_Msg_Header {bool is_encrypted; uint_2 size_low; uint_9d3
    size_high} with size = size_low + (size_high << 2) (docs/WIRE_FORMAT.md 2.1, 2.2, 2.4;
    bits are written LSB first)."""
    bits = []

    def w(v, n):
        bits.extend((v >> i) & 1 for i in range(n))

    w(1 if is_encrypted else 0, 1)
    w(size & 3, 2)
    v, remaining = size >> 2, 9  # uint_9d3
    w(1 if v else 0, 1)
    if v:
        while True:
            c = min(3, remaining)
            w(v & ((1 << c) - 1), c)
            v >>= c
            remaining -= c
            if remaining == 0:
                break
            w(1 if v else 0, 1)
            if v == 0:
                break
    bits += [0] * (-len(bits) % 8)
    return bytes(sum(bits[i + k] << k for k in range(8)) for i in range(0, len(bits), 8))


def payload(size):
    """FRAMING.md section 5: p[i] = (i*37 + 1) & 0xff."""
    return bytes((i * 37 + 1) & 0xFF for i in range(size))


def lcg_bytes(n, seed=0x12345678):
    """Same rule as lcg_1000 in crc32.txt."""
    x = seed
    out = bytearray()
    for _ in range(n):
        x = (x * 1664525 + 1013904223) & 0xFFFFFFFF
        out.append(x >> 24)
    return bytes(out)


# carachangprotocol/FRAMING.md section 5: (is_encrypted, size, header, n, CRC trailer)
FRAMING_TABLE = [
    (False, 1, "02", 2, "95"),
    (False, 3, "06", 4, "71"),
    (False, 11, "2E", 12, "2D"),
    (False, 12, "38", 13, "B1"),
    (True, 12, "39", 13, "D3"),
    (False, 13, "3A", 14, "EF"),
    (False, 31, "7E", 32, "F9 AF"),
    (False, 32, "88 01", 34, "F4 3B"),
    (True, 64, "89 02", 66, "FD 30"),
    (False, 300, "B8 19", 302, "C7 15"),
]
# additional frames: header only (n = 1), the CRC-16 side of the boundary with a 1-byte
# header (n = 15, 16), the largest 1-byte header frame (n = 32) and the largest size
EXTRA_FRAMES = [(False, 0), (False, 14), (True, 14), (False, 15), (True, 31), (False, 2047)]

ASCII = [
    ("check", "123456789"),
    ("ascii_a", "a"),
    ("ascii_abc", "abc"),
    ("ascii_alphabet", "abcdefghijklmnopqrstuvwxyz"),
]
HEX = [
    ("empty", b""),
    ("byte_00", b"\x00"),
    ("byte_ff", b"\xff"),
    ("byte_01", b"\x01"),
    ("byte_80", b"\x80"),
    ("zeros_4", b"\x00" * 4),
    ("ones_4", b"\xff" * 4),
    ("zeros_32", b"\x00" * 32),
    ("ones_32", b"\xff" * 32),
    ("quick_brown_fox", b"The quick brown fox jumps over the lazy dog"),
    ("counting_256", bytes(range(256))),
    ("lcg_1000", lcg_bytes(1000)),
]


def frame_name(is_encrypted, size):
    return f"{'enc' if is_encrypted else 'plain'}_{size}"


def frames():
    """(name, header, payload, trailer) of all frame records, FRAMING.md table first."""
    out = []
    for enc, size, header, n, trailer in FRAMING_TABLE:
        h, p = xtx_header(enc, size), payload(size)
        assert h == bytes.fromhex(header), (enc, size, h.hex())
        assert len(h) + len(p) == n, (enc, size)
        t = transmission_trailer(h + p)
        assert t == bytes.fromhex(trailer), (enc, size, t.hex())
        if n >= 15:
            assert t == crc16_ibm3740(h + p).to_bytes(2, "little")
        out.append((frame_name(enc, size), h, p, t))
    for enc, size in EXTRA_FRAMES:
        h, p = xtx_header(enc, size), payload(size)
        out.append((frame_name(enc, size), h, p, transmission_trailer(h + p)))
    names = [f[0] for f in out]
    assert len(set(names)) == len(names)
    ns = {len(h) + len(p) for _, h, p, _ in out}
    assert {1, 14, 15} <= ns, ns
    return out


def render():
    assert crc8_autosar(b"123456789") == 0xDF and crc16_ibm3740(b"123456789") == 0x29B1
    lines = [
        "# CRC-8/AUTOSAR + CRC-16/IBM-3740 test vectors -- GENERATED by",
        "# golden/vectors/crc8_16_gen.py (bit-wise reference), do not edit (see golden/README.md,",
        "# section CRC-8/16 vectors)",
        "# crc8: poly 0x2f, init 0xff, xorout 0xff; crc16: poly 0x1021, init 0xffff, xorout 0;",
        "# no reflection",
        "# hex   <name> <input_hex> <crc8> <crc16>     input lower-case hex, '-' = empty input",
        "# ascii <name> <input>     <crc8> <crc16>     input = the ASCII bytes of the text",
        "# frame <name> <header_hex> <payload_hex> <trailer_hex>",
        "#       transmission CRC (carachangprotocol FRAMING.md section 2) over header + payload,",
        "#       n = header + payload bytes: n <= 14 -> 1 byte CRC-8, n >= 15 -> 2 bytes CRC-16",
        "#       little-endian (low byte first); payload '-' = empty; header = bitis XTX_Msg_Header",
        "#       {bool is_encrypted; uint_2 size_low; uint_9d3 size_high}, payload p[i] =",
        "#       (i*37 + 1) & 0xff; name = plain|enc _ <payload size>. The first",
        f"#       {len(FRAMING_TABLE)} frames are the test vectors of FRAMING.md section 5.",
        "# <crc8>: 2, <crc16>: 4 lower-case hex digits",
        "# lcg_1000: x = 0x12345678; 1000 times: x = (x * 1664525 + 1013904223) mod 2^32,",
        "#           emit byte x >> 24",
    ]
    for name, text in ASCII:
        assert " " not in text and text.isascii() and text.isprintable()
        d = text.encode()
        lines.append(f"ascii {name} {text} {crc8_autosar(d):02x} {crc16_ibm3740(d):04x}")
    for name, data in HEX:
        lines.append(f"hex {name} {data.hex() or '-'} {crc8_autosar(data):02x} {crc16_ibm3740(data):04x}")
    for name, h, p, t in frames():
        lines.append(f"frame {name} {h.hex()} {p.hex() or '-'} {t.hex()}")
    return "\n".join(lines) + "\n"


def render_dart(text):
    assert "'''" not in text
    return (
        "// GENERATED by golden/vectors/crc8_16_gen.py from golden/vectors/crc8_16.txt. Do not edit.\n"
        "// ignore_for_file: lines_longer_than_80_chars\n"
        "\n"
        "/// Content of golden/vectors/crc8_16.txt, embedded because the web platforms\n"
        "/// have no file I/O (regenerate with `python3 golden/vectors/crc8_16_gen.py`).\n"
        "const crc8Crc16Vectors = r'''\n"
        f"{text}"
        "''';\n"
    )


def main():
    here = pathlib.Path(__file__).resolve().parent
    text = render()
    outputs = [
        (here / "crc8_16.txt", text),
        (here.parent / "dart" / "test" / "crc8_16_vectors.g.dart", render_dart(text)),
    ]
    if sys.argv[1:] == ["--check"]:
        for out, content in outputs:
            if not out.exists() or out.read_text() != content:
                print(f"{out} is out of date, run golden/vectors/crc8_16_gen.py")
                sys.exit(1)
        return
    if sys.argv[1:]:
        print(__doc__)
        sys.exit(2)
    for out, content in outputs:
        out.write_text(content)
        print(f"wrote {out}")


if __name__ == "__main__":
    main()
