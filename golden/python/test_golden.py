"""Golden vector tests of the Python bindings (golden/README.md).

Runs the checks of golden/vectors/manifest.txt against the module generated from
golden/schema/golden.bitis (built by golden/python/build.sh), plus spot checks of decoded
values, the encoder, the exceptions and the type stub. The module name is taken from
BITIS_PY_MODULE (default `golden`).

    BITIS_PY_MODULE=golden python -m pytest golden/python
"""
import importlib
import math
import os
import pathlib
import shutil
import subprocess
import sys

import pytest

mod = importlib.import_module(os.environ.get("BITIS_PY_MODULE", "golden"))

HERE = pathlib.Path(__file__).resolve().parent
MANIFEST = HERE.parent / "vectors" / "manifest.txt"
ERROR_KINDS = {"UnexpectedEnd", "InvalidValue", "InvalidDiscriminant", "TrailingData"}


def unhex(s):
    return b"" if s == "-" else bytes.fromhex(s)


def load_manifest():
    """Records by type; unknown line types are ignored (newer manifests may add some)."""
    recs = {"valid": [], "noncanon": [], "invalid": [], "min_bits": [], "max_bits": []}
    schema_hash = None
    for n, line in enumerate(MANIFEST.read_text().splitlines(), 1):
        if line.startswith("# schema:") and "schema hash" in line:
            schema_hash = int(line.rsplit(" ", 1)[1], 16)
        if not line or line.startswith("#"):
            continue
        f = line.split(" ")
        if f[0] in recs:
            recs[f[0]].append(pytest.param(*f[1:], id=f[1] if f[0] in ("valid", "noncanon", "invalid") else f[1]))
    return recs, schema_hash


RECORDS, SCHEMA_HASH = load_manifest()


def msg_type(name):
    return getattr(mod, name)


def decode_error_kind(exc_info):
    e = exc_info.value
    assert type(e) is mod.DecodeError, type(e)
    assert isinstance(e, ValueError)
    return e.kind


# ****************************************************************************
# manifest records

def test_manifest_has_records():
    assert all(len(RECORDS[k]) > 0 for k in ("valid", "noncanon", "invalid", "min_bits"))


def test_schema_stamp():
    assert mod.BITIS_SCHEMA_HASH == SCHEMA_HASH
    assert isinstance(mod.BITIS_VERSION, str) and mod.BITIS_VERSION.count(".") == 2


@pytest.mark.parametrize("case,ty,total_bits,hexdata", RECORDS["valid"])
def test_valid(case, ty, total_bits, hexdata):
    T, data, bits = msg_type(ty), unhex(hexdata), int(total_bits)
    nbytes = (bits + 7) // 8
    v, b, nb = T.try_deserialize(data)
    assert (b, nb) == (bits, nbytes)
    assert T.try_serialize(v) == (data, bits, nbytes)
    assert v.serialize() == (data, bits, nbytes)
    assert T.try_deserialize_exact(data).try_serialize()[0] == data
    assert T.deserialize(data).serialize()[0] == data
    # trailing data: ignored by the default decoders, rejected by the exact decoder
    longer = data + b"\xa5\x5a"
    v, b, nb = T.try_deserialize(longer)
    assert (b, nb) == (bits, nbytes)
    assert v.try_serialize()[0] == data
    assert T.deserialize(longer).try_serialize()[0] == data
    with pytest.raises(mod.DecodeError) as ei:
        T.try_deserialize_exact(longer)
    assert decode_error_kind(ei) == "TrailingData"


@pytest.mark.parametrize("case,ty,total_bits,input_hex,canonical_hex", RECORDS["noncanon"])
def test_noncanon(case, ty, total_bits, input_hex, canonical_hex):
    T, data, bits = msg_type(ty), unhex(input_hex), int(total_bits)
    v, b, _ = T.try_deserialize(data)
    assert b == bits
    assert v.try_serialize()[0] == unhex(canonical_hex)
    assert T.try_deserialize_exact(data).try_serialize()[0] == unhex(canonical_hex)


@pytest.mark.parametrize("case,ty,error,mode,hexdata", RECORDS["invalid"])
def test_invalid(case, ty, error, mode, hexdata):
    assert error in ERROR_KINDS
    T, data = msg_type(ty), unhex(hexdata)
    decoders = [T.try_deserialize, T.deserialize] if mode == "default" else [T.try_deserialize_exact]
    for decode in decoders:
        with pytest.raises(mod.DecodeError) as ei:
            decode(data)
        assert decode_error_kind(ei) == error
        assert str(ei.value).startswith(error + ": ")


@pytest.mark.parametrize("ty,bits", RECORDS["min_bits"])
def test_min_bits(ty, bits):
    assert msg_type(ty).min_bits() == int(bits)


@pytest.mark.parametrize("ty,bits", RECORDS["max_bits"])
def test_max_bits(ty, bits):
    assert msg_type(ty).max_bits() == (None if bits == "unbounded" else int(bits))


# ****************************************************************************
# spot checks: decoded values

def case_bytes(case):
    for kind in ("valid", "noncanon"):
        for p in RECORDS[kind]:
            if p.values[0] == case:
                return unhex(p.values[3])
    raise KeyError(case)


def decode(ty, case):
    return msg_type(ty).try_deserialize_exact(case_bytes(case))


def test_decoded_ints():
    m = decode("Ints", "ints_min")
    assert (m.val_i1, m.val_i7, m.val_i16, m.val_i32) == (-1, -127, -32767, -2147483647)
    assert (m.val_i33, m.val_i64) == (-8589934591, -9223372036854775807)
    m = decode("Ints", "ints_max")
    assert (m.val_u33, m.val_u63, m.val_u64) == ((1 << 33) - 1, (1 << 63) - 1, (1 << 64) - 1)
    m = decode("DynInts", "dynints_max")
    assert (m.val_u8d3, m.val_u64d7) == (255, (1 << 64) - 1)
    m = decode("DynInts", "noncanon_dynints_nonminimal")
    assert (m.val_u8d3, m.val_u32d4, m.val_i16d5) == (5, 5, -3)


def test_decoded_floats():
    m = decode("Floats", "floats_negzero")
    assert m.val_f64 == 0.0 and math.copysign(1, m.val_f64) < 0 and math.copysign(1, m.val_f32) < 0
    m = decode("Floats", "floats_nan")
    assert math.isnan(m.val_f32) and math.isnan(m.val_f64)


def test_decoded_fixed_points():
    m = decode("FixedPoints", "fixedpoints_gnss")
    assert abs(m.lat - 51.756349) <= 6e-9 and abs(m.lng - 14.332868) <= 6e-9
    assert m.fp2 == 1.0 and abs(m.fp53 - 123456.789) <= 1e-9
    m = decode("FixedPoints", "fixedpoints_min")
    assert (m.fp2, m.fp10, m.lat, m.lng, m.fp53) == (0.0, -2.0, -90.0, -180.0, -1e6)
    # Overflow reads back as max + 1, Underflow as min - 1 (see the .pyi)
    m = decode("FixedPoints", "fixedpoints_default")
    assert (m.fp2, m.fp10, m.lat, m.lng, m.fp53) == (2.0, 4.0, 91.0, 181.0, 1000001.0)
    assert mod.FixedPoints.default().try_serialize()[0] == case_bytes("fixedpoints_default")
    m = decode("FixedPoints", "fixedpoints_underflow")
    assert (m.fp2, m.fp10, m.lat, m.lng, m.fp53) == (-1.0, -3.0, -91.0, -181.0, -1000001.0)


def test_decoded_texts_containers_enums():
    m = decode("Texts", "texts_values")
    assert m.text == "Hello, bitis!" and m.data == bytes([0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF])
    m = decode("Containers", "containers_full")
    assert m.opt_color == mod.Color.Yellow and m.opt_text == ""
    assert m.fixed_ints == [-127, 0, 127]
    assert (len(m.dyn_leaves), m.dyn_leaves[2].val, m.dyn_leaves[0].tag) == (3, -63, "deep")
    m = decode("Containers", "containers_default")
    assert m.opt_u7 is None and m.opt_leaf is None and m.dyn_ints == []
    m = decode("Enums", "enums_all")
    assert (m.color, m.level, len(m.colors), m.colors[5]) == (mod.Color.Yellow, mod.Level.High, 6, mod.Color.Yellow)
    assert m.color != mod.Color.Red and int(m.color) == 5
    assert {mod.Color.Yellow: 1}[m.color] == 1


def test_decoded_oneof_nested_bounds():
    m = decode("Choice", "choice_text")
    assert m.value_oo == mod.OO_Choice_ValueEnum.Text and m.value_text == "oneof text" and m.value_flag is None
    m = decode("Choice", "choice_list")
    assert m.value_oo == mod.OO_Choice_ValueEnum.List and m.value_list.items == [-1, 0, 1, 127, -127]
    m = decode("Choice", "choice_big")
    assert m.value_big == -2147483647
    m = decode("Nested", "nested_deep")
    assert (m.mid.id, m.mid.leaf.val, m.mid.leaf.tag, m.mid.extra.tag, m.after) == (99, -42, "level3", "extra", True)
    m = decode("Bounds", "bounds_max")
    assert (m.name, m.blob, m.tags, m.note) == ("12345678", bytes([0, 0x80, 0xFF, 0x7F]), ["ab", "cd", "ef"], "note!")
    assert [(i.val, i.label) for i in m.items] == [(15, "xyz"), (9, "~~~")]


# ****************************************************************************
# spot checks: encoder

def encode(m):
    return m.try_serialize()[0]


def test_encode_fixed_points():
    m = mod.FixedPoints(0.5, -1.845739471106758, -1.5684636309743063e-05, -1.5684636309743063e-05, -3.32365743815901e-07)
    assert encode(m) == case_bytes("fixedpoints_ties")
    m = mod.FixedPoints(0.49999999999999994, -1.0, 51.756349, 14.332868, 123456.789)
    assert encode(m) == case_bytes("fixedpoints_gnss")
    m = mod.FixedPoints(float("nan"), -2.001, -90.000001, float("-inf"), -1000000.001)
    assert encode(m) == case_bytes("fixedpoints_underflow")


def test_encode_values():
    m = mod.Ints.default()
    m.val_i1, m.val_i7, m.val_i16, m.val_i32 = -1, -127, -32767, -2147483647
    m.val_i33, m.val_i64 = -8589934591, -9223372036854775807
    assert encode(m) == case_bytes("ints_min")
    assert encode(mod.Texts("Hello, bitis!", bytes([0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF]))) == case_bytes("texts_values")
    assert mod.Empty().try_serialize() == (b"", 0, 0)
    c = mod.Choice(mod.OO_Choice_Value.new_leaf(mod.Leaf(-5, "x")), True)
    assert c.value_oo == mod.OO_Choice_ValueEnum.Leaf and c.value_leaf.val == -5
    c.value_num = 2.0
    assert c.value_oo == mod.OO_Choice_ValueEnum.Num
    assert mod.Choice.try_deserialize_exact(encode(c)).value_num == 2.0


def test_encode_out_of_range():
    for ty, attr, val in [("Ints", "val_i64", -(1 << 63)), ("Ints", "val_i16", -(1 << 15)),
                          ("DynInts", "val_i16d5", -(1 << 15)), ("Ints", "val_u7", 128)]:
        m = msg_type(ty).default()
        setattr(m, attr, val)
        with pytest.raises(mod.EncodeError) as ei:
            m.try_serialize()
        assert type(ei.value) is mod.EncodeError and isinstance(ei.value, ValueError)
        assert ei.value.kind == "ValueOutOfRange"
        m.serialize()  # legacy: truncated, no exception
    # values that do not fit the Rust field type (u8..u64, i8..i64) are rejected when setting
    m = mod.Ints.default()
    with pytest.raises(OverflowError):
        m.val_u7 = 1 << 70
    # a non-ASCII string: EncodeError from try_serialize, empty string from the legacy serialize
    t = mod.Texts("hä", b"")
    with pytest.raises(mod.EncodeError) as ei:
        t.try_serialize()
    assert ei.value.kind == "ValueOutOfRange"
    assert t.serialize()[0] == mod.Texts("", b"").serialize()[0]
    n = mod.Nested.default()
    n.mid.leaf.tag = "ä"
    with pytest.raises(mod.EncodeError):
        n.try_serialize()


def test_bounds():
    bounds = mod.Bounds.try_deserialize_exact(case_bytes("bounds_max"))
    assert encode(bounds) == case_bytes("bounds_max")
    # longer than the maximum length: EncodeError (a ValueError) when constructing / setting
    with pytest.raises(mod.EncodeError) as ei:
        bounds.name = "12345678+"
    assert ei.value.kind == "ValueOutOfRange"
    with pytest.raises(mod.EncodeError):
        mod.Bounds("123456789", b"", [], None, [])
    with pytest.raises(ValueError):
        bounds.items = [mod.BoundsItem(1, None)] * 3
    assert encode(bounds) == case_bytes("bounds_max")


# ****************************************************************************
# type stub (.pyi): a usage script must pass mypy --strict

USAGE = '''from __future__ import annotations
from {m} import (Choice, Color, Containers, DecodeError, EncodeError, Leaf, OO_Choice_Value,
                 OO_Choice_ValueEnum, Texts, BITIS_SCHEMA_HASH, BITIS_VERSION)

def roundtrip(data: bytes) -> tuple[bytes, int]:
    c, bits, nbytes = Containers.try_deserialize(data)
    opt: int | None = c.opt_u7
    color: Color | None = c.opt_color
    leaves: list[Leaf] = c.dyn_leaves
    text: str = leaves[0].tag if leaves else ""
    out, b, nb = c.try_serialize()
    legacy: tuple[bytes, int, int] = c.serialize()
    return out, bits + nbytes + b + nb + (opt or 0) + len(text) + int(color or Color.Red) + legacy[1]

def errors(data: bytes) -> str:
    try:
        Texts.try_deserialize_exact(data)
    except DecodeError as e:
        return e.kind
    try:
        Texts("x", b"\\x00").try_serialize()
    except EncodeError as e:
        return e.kind
    return BITIS_VERSION + str(BITIS_SCHEMA_HASH)

def oneof() -> float | None:
    c = Choice(OO_Choice_Value.new_num(1.5), True)
    if c.value_oo == OO_Choice_ValueEnum.Num:
        c.value_flag = True
    m: int = Choice.min_bits()
    x: int | None = Choice.max_bits()
    return c.value_num if m and x is None else None
'''


def test_stub_mypy_strict(tmp_path):
    mypy = shutil.which("mypy", path=os.path.dirname(sys.executable)) or shutil.which("mypy")
    if mypy is None:
        pytest.skip("mypy not installed")
    script = tmp_path / "usage.py"
    script.write_text(USAGE.format(m=mod.__name__))
    r = subprocess.run([mypy, "--strict", "--no-incremental", str(script)],
                       capture_output=True, text=True, cwd=tmp_path)
    assert r.returncode == 0, r.stdout + r.stderr


def test_stub_matches_runtime():
    """mypy stubtest: every name, signature and decorator of the stub matches the module (the
    pyo3 variant subclasses of the oneof classes, e.g. OO_Choice_Value.Text, are not in the stub)."""
    pytest.importorskip("mypy.stubtest")
    r = subprocess.run([sys.executable, "-m", "mypy.stubtest", "--ignore-missing-stub", "--ignore-disjoint-bases",
                        mod.__name__], capture_output=True, text=True)
    assert r.returncode == 0, r.stdout + r.stderr
