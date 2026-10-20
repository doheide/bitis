# Changelog

## 0.11.0 (2026-10-20)

The wire format is unchanged: data encoded with 0.10.x decodes with 0.11.0 and
vice versa, and schemas without the new length bounds keep their meaning. The
format is now specified in `docs/WIRE_FORMAT.md` (normative). Golden vectors
(`golden/`) check that the Rust, C++, Python and Dart runtimes are byte
identical (`golden/run_all.sh`).

### Safe decoding of untrusted input (all runtimes)
- Truncated input fails with `UnexpectedEnd` instead of decoding zero bits; counts
  of strings, binaries and lists are checked against the remaining input before
  anything is allocated; unknown enum / oneof indices fail with
  `InvalidDiscriminant`; out-of-range values and non-ASCII strings fail with
  `InvalidValue`. Encoders no longer hang on out-of-range values.
- New fallible API next to the old one: `try_serialize`, `try_deserialize`,
  `try_deserialize_exact` (fails with `TrailingData` on extra bytes) in Rust,
  C++, Python and Dart. Error kinds are the same in every runtime.
- Rust: `DecodeError` / `EncodeError`, `Bides::error()` / `Biseri::error()`,
  `try_new`, `MIN` / `MAX` for integer wrappers, `value()`, `is_overflow()`,
  `is_underflow()` for fixed point.

### Compiler
- Validation with `file:line:col` errors (ranges, duplicate and reserved names,
  names that collide with generated members), deterministic output, doc comment
  fix.
- Schema stamp in the generated code: compiler version and a 64-bit schema hash
  (`BITIS_VERSION`, `BITIS_SCHEMA_HASH`; spec section 5).
- Maximum lengths: `astr_dY[M]`, `binary_dY[M]`, `repeated_dyn_Y[M] T`
  (1 ≤ M ≤ 65535, spec 2.16). Longer values are `InvalidValue` when decoding and
  `ValueOutOfRange` when encoding. C++ uses heap-free containers for them
  (`BoundedAString`, `BoundedBinary`, `BoundedDynArray`).
- `min_bits` / `max_bits` of every type in every runtime (spec section 4).
- New target `-l dart`.

### C++ runtime
- Bounds-checked decoding, sticky `BitisError`, non-owning decoding from
  `(const uint8_t*, size_t)`, oneof values in properly typed tagged storage.
- Footprint: `BITIS_NO_PRINT` (all `print()` become empty, no name strings),
  one static attribute name table per message type, shared non-template core
  code, compact fixed point storage. Measured on the ESP32 firmware
  (`sm_sercon.cpp`, text size, -Og / -Os): 62.4 / 41.0 KB with 0.10 →
  52.9 / 35.0 KB, with `BITIS_NO_PRINT` 34.2 / 22.6 KB. Message objects are up
  to ~5× smaller (e.g. `DTS_Msg_GNSSPosition` 224 → 40 bytes, `DTS_MsgWrapper`
  248 → 56 bytes on ESP32).
- Encoding into a caller buffer: `try_serialize(msg, buf, cap, &size)` with the new
  error `BitisError::BufferTooSmall`; compile-time `bitis_max_bits<T>()` /
  `bitis_max_bytes<T>()` (`BITIS_UNBOUNDED` if unbounded).
- Include guard, C++17, no exceptions, no RTTI; tests run plain and with
  ASan/UBSan.

### Python
- Exceptions `DecodeError` / `EncodeError` (subclasses of `ValueError`,
  attribute `kind`), `try_*` functions, `min_bits()` / `max_bits()`, complete
  `.pyi` stubs (checked with mypy), hashable enums.
- The compiler pins pyo3 0.29 with `abi3-py38` and sets the package version to
  the bitis version: one wheel `<name>-<version>-cp38-abi3-<platform>.whl` for
  CPython ≥ 3.8.

### Dart (new)
- Runtime package `dart/bitis` (pure Dart, web-safe: VM, dart2js, dart2wasm) and
  code generation with immutable message classes, JSON mapping and the schema
  stamp.

### CRC-32 helper (new, not part of the wire format)
- CRC-32/ISO-HDLC (= zlib) in every runtime: Rust `bitis_lib::crc32` / `Crc32`,
  C++ `cpp_lib/bitis_crc32.h` (`bitis_crc32()` / `BitisCrc32`), Dart
  `crc32` / `Crc32`, Python `crc32` / `Crc32`. Same as the ESP32 ROM call
  `esp_rom_crc32_le(0, buf, len)`.

### Migration notes
Existing code keeps compiling except for the points below.

C++:
- Fixed point fields no longer have the data members `value` / `state` (they
  store the raw wire value). Use `value()` / `get_value()`, `state()` /
  `get_state()`, `has_value()`, `is_overflow()`, `is_underflow()`, `set(v)`,
  `set_overflow()`, `set_underflow()`, `raw()`. Constructors are unchanged.
- Messages no longer have the members `str_<attr>`; `msg_attr` is a static
  table (absent with `BITIS_NO_PRINT`).
- `BitisAString` keeps a terminating NUL in its byte vector, so `get()` /
  `c_str()` are proper C strings. `get_u8_vec()` therefore includes that NUL;
  use `size()` for the length.
- Oneofs: `set_oo<OO_X>(v)` and `get_oo<OO_X>()` (pointer, `nullptr` if another
  element is selected) are unchanged. The value is now stored as a real object
  of its type: a pointer from `get_oo` stays valid only until the oneof is
  changed.
- Legacy `deserialize()` returns a default constructed value on *any* error
  (before: truncated input decoded zero bits). Prefer `try_deserialize_exact()`.
- `min_bits`, `max_bits` and some C++/Python/Dart keywords are reserved as
  attribute names; the compiler reports collisions.

Python:
- `deserialize()` raises `DecodeError` instead of a bare `Exception` (both are
  caught by `except Exception`). Wheel file names change (abi3, real version):
  build scripts must not hard-code `-0.1.0-cp312-cp312-`.
- The next Python compile updates the project's `Cargo.toml` (pyo3 0.29 with
  abi3, `strum_macros`, version).

Rust:
- `max_bits()` returns `Some` for more types (dyn integers, enums, oneofs,
  messages without unbounded fields).
