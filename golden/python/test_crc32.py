"""CRC-32 helper of a generated bitis Python module (crc32 / Crc32, implemented by the Rust
runtime) against golden/vectors/crc32.txt and zlib.crc32.

The module under test is any module generated with `bitis compile -l python` (the CRC
functions do not depend on the schema): its name is taken from BITIS_PY_MODULE (default
`golden`); the tests are skipped if it cannot be imported.

    BITIS_PY_MODULE=<module> python -m pytest golden/python/test_crc32.py
"""
import os
import pathlib
import zlib

import pytest

mod = pytest.importorskip(os.environ.get("BITIS_PY_MODULE", "golden"))
crc32 = mod.crc32
Crc32 = mod.Crc32

VECTORS = pathlib.Path(__file__).resolve().parent.parent / "vectors" / "crc32.txt"


def load_vectors():
    out = []
    for line in VECTORS.read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        kind, name, data, crc = line.split(" ")
        if kind == "hex":
            data = b"" if data == "-" else bytes.fromhex(data)
        elif kind == "ascii":
            data = data.encode("ascii")
        else:
            raise ValueError(f"unknown record kind: {line}")
        out.append((name, data, int(crc, 16)))
    assert len(out) >= 16
    return out


@pytest.mark.parametrize("name,data,crc", load_vectors(), ids=lambda v: v if isinstance(v, str) else "")
def test_vector(name, data, crc):
    assert crc32(data) == crc
    assert zlib.crc32(data) == crc
    n = len(data)
    cuts = {0, 1, n // 2, max(n - 1, 0), n} | (set(range(n + 1)) if n <= 64 else set())
    for i in sorted(c for c in cuts if c <= n):
        c = Crc32()
        c.update(data[:i])
        c.update(data[i:])
        assert c.finish() == crc, f"split at {i}"
        assert c.value == crc
    c = Crc32()
    for b in data:
        c.update(bytes([b]))
    assert c.value == crc
    c.reset()
    assert c.value == 0
    c.update(data)
    assert c.finish() == crc


def test_check_value_and_input_types():
    assert crc32(b"123456789") == 0xCBF43926
    assert crc32(b"") == 0
    assert crc32(bytearray(b"123456789")) == 0xCBF43926
    assert crc32(memoryview(b"123456789")) == 0xCBF43926
    assert crc32(list(b"123456789")) == 0xCBF43926
    assert Crc32().value == 0
    with pytest.raises(TypeError):
        crc32("123456789")
    with pytest.raises((TypeError, OverflowError)):
        crc32([256])


def test_same_as_zlib_incremental():
    data = bytes((i * 7 + 3) & 0xFF for i in range(5000))
    c = Crc32()
    z = 0
    for k in range(0, len(data), 333):
        c.update(data[k:k + 333])
        z = zlib.crc32(data[k:k + 333], z)
        assert c.value == z
    assert c.finish() == zlib.crc32(data) == crc32(data)
