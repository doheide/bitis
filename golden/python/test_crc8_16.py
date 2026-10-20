"""CRC-8/AUTOSAR, CRC-16/IBM-3740 and transmission CRC helpers of a generated bitis Python
module (implemented by the Rust runtime) against golden/vectors/crc8_16.txt.

The module under test is any module generated with `bitis compile -l python` (the CRC
functions do not depend on the schema): its name is taken from BITIS_PY_MODULE (default
`golden`); the tests are skipped if it cannot be imported.

    BITIS_PY_MODULE=<module> python -m pytest golden/python/test_crc8_16.py
"""
import os
import pathlib

import pytest

mod = pytest.importorskip(os.environ.get("BITIS_PY_MODULE", "golden"))

VECTORS = pathlib.Path(__file__).resolve().parent.parent / "vectors" / "crc8_16.txt"


def unhex(s):
    return b"" if s == "-" else bytes.fromhex(s)


def load():
    vectors, frames = [], []
    for line in VECTORS.read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        kind, name, a, b, c = line.split(" ")
        if kind == "frame":
            frames.append((name, unhex(a), unhex(b), unhex(c)))
            continue
        if kind == "hex":
            data = unhex(a)
        elif kind == "ascii":
            data = a.encode("ascii")
        else:
            raise ValueError(f"unknown record kind: {line}")
        assert len(b) == 2 and len(c) == 4
        vectors.append((name, data, int(b, 16), int(c, 16)))
    assert len(vectors) >= 16 and len(frames) >= 11
    return vectors, frames


VECTOR_LIST, FRAME_LIST = load()


def ids(v):
    return v if isinstance(v, str) else ""


@pytest.mark.parametrize("name,data,crc8,crc16", VECTOR_LIST, ids=ids)
def test_vector(name, data, crc8, crc16):
    assert mod.crc8_autosar(data) == crc8
    assert mod.crc16_ibm3740(data) == crc16
    n = len(data)
    cuts = {0, 1, n // 2, max(n - 1, 0), n} | (set(range(n + 1)) if n <= 64 else set())
    for i in sorted(c for c in cuts if c <= n):
        c8, c16 = mod.Crc8Autosar(), mod.Crc16Ibm3740()
        for c in (c8, c16):
            c.update(data[:i])
            c.update(data[i:])
        assert (c8.finish(), c16.finish()) == (crc8, crc16), f"split at {i}"
        assert (c8.value, c16.value) == (crc8, crc16)
    c8, c16 = mod.Crc8Autosar(), mod.Crc16Ibm3740()
    for b in data:
        c8.update(bytes([b]))
        c16.update(bytes([b]))
    assert (c8.value, c16.value) == (crc8, crc16)
    c8.reset()
    c16.reset()
    assert (c8.value, c16.value) == (0x00, 0xFFFF)
    c8.update(data)
    c16.update(data)
    assert (c8.finish(), c16.finish()) == (crc8, crc16)


@pytest.mark.parametrize("name,header,payload,trailer", FRAME_LIST, ids=ids)
def test_frame(name, header, payload, trailer):
    frame = header + payload
    n = len(frame)
    assert payload == bytes((i * 37 + 1) & 0xFF for i in range(len(payload)))
    assert mod.transmission_crc_len(n) == len(trailer)
    crc = mod.transmission_crc(frame)
    if n <= 14:
        assert trailer == bytes([crc]) == bytes([mod.crc8_autosar(frame)])
    else:
        assert trailer == crc.to_bytes(2, "little") == mod.crc16_ibm3740(frame).to_bytes(2, "little")
    full = mod.append_transmission_crc(frame)
    assert isinstance(full, bytes) and full == frame + trailer
    assert mod.append_transmission_crc(bytearray(frame)) == full
    assert mod.verify_transmission_crc(full)
    assert mod.strip_transmission_crc(full) == frame
    assert mod.strip_transmission_crc(memoryview(full)) == frame
    for bit in range(len(full) * 8):
        bad = bytearray(full)
        bad[bit // 8] ^= 1 << (bit % 8)
        assert not mod.verify_transmission_crc(bad), f"bit {bit}"
        with pytest.raises(ValueError, match="mismatch"):
            mod.strip_transmission_crc(bytes(bad))


def test_check_values_and_input_types():
    assert mod.crc8_autosar(b"123456789") == 0xDF
    assert mod.crc16_ibm3740(b"123456789") == 0x29B1
    assert mod.crc8_autosar(b"") == 0x00
    assert mod.crc16_ibm3740(b"") == 0xFFFF
    for conv in (bytearray, memoryview, list):
        assert mod.crc8_autosar(conv(b"123456789")) == 0xDF
        assert mod.crc16_ibm3740(conv(b"123456789")) == 0x29B1
    assert mod.Crc8Autosar().value == 0 and mod.Crc16Ibm3740().value == 0xFFFF
    with pytest.raises(TypeError):
        mod.crc16_ibm3740("123456789")
    with pytest.raises((TypeError, OverflowError)):
        mod.crc8_autosar([256])


def test_transmission_lengths():
    assert [mod.transmission_crc_len(n) for n in (0, 14, 15, 2049)] == [1, 1, 2, 2]
    with pytest.raises(OverflowError):
        mod.transmission_crc_len(-1)
    l15 = mod.append_transmission_crc(bytes([7] * 14))
    l17 = mod.append_transmission_crc(bytes([7] * 15))
    assert (len(l15), len(l17)) == (15, 17)
    assert mod.verify_transmission_crc(l15) and mod.verify_transmission_crc(l17)
    # L = 16 is invalid, also with a CRC-16 (little-endian) that matches its first 14 bytes
    l16 = bytes([7] * 14) + mod.crc16_ibm3740(bytes([7] * 14)).to_bytes(2, "little")
    assert not mod.verify_transmission_crc(l16)
    with pytest.raises(ValueError, match="length 16"):
        mod.strip_transmission_crc(l16)
    # empty input / n = 0
    assert not mod.verify_transmission_crc(b"")
    with pytest.raises(ValueError, match="length 0"):
        mod.strip_transmission_crc(b"")
    assert mod.append_transmission_crc(b"") == b"\x00"
    assert mod.strip_transmission_crc(b"\x00") == b""
