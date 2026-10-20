# bitis 0.11 development plan (branch `dev/v0.11`)

Goal: safe decoding of untrusted input in all runtimes, compiler fixes, and a new
Dart target. The wire format (docs/WIRE_FORMAT.md) does NOT change.

Compatibility rule for 0.11: **no breaking API changes**. Existing functions keep
their signatures; new fallible functions are added next to them. Behaviour
changes are limited to bug fixes (e.g. truncated input now fails instead of
decoding zero bits; encoders no longer hang). A later breaking release (Result-
based traits, crate split) will be coordinated with the backend/firmware agents.

## Rust runtime (src/lib_impl/berde.rs, src/lib.rs, bitis-macros)

- `pub enum DecodeError { UnexpectedEnd, InvalidValue, InvalidDiscriminant, TrailingData }`
- `pub enum EncodeError { ValueOutOfRange }`
  (both `Debug, Clone, Copy, PartialEq, Eq`, `Display`, `std::error::Error`)
- Traits keep returning `Option`; the reason of a failure is recorded in a
  side channel (`Bides::error() -> Option<DecodeError>`, first error wins;
  `Biseri::error() -> Option<EncodeError>`).
- New top-level functions:
  - `try_serialize<T>(&T) -> Result<(Vec<u8>, BiserSizes), EncodeError>`
  - `try_deserialize<T>(&[u8]) -> Result<(T, BiserSizes), DecodeError>`
  - `try_deserialize_exact<T>(&[u8]) -> Result<T, DecodeError>`
- Legacy `serialize` keeps silently truncating out-of-range values (documented),
  but never hangs. Legacy `deserialize` returns `None` on every error.
- `BiserdiTrait` gets `fn max_bits() -> Option<u64>` with default `None`
  (`Some(n)` for fixed-size types and messages consisting only of them).
- Integer wrappers: `try_new`, `MIN`/`MAX` constants. Fixed point: explicit
  state accessors (`value() -> Option<f64>`, `is_overflow()`, `is_underflow()`).

## C++ runtime (cpp_lib/bitis_lib.h)

- `enum class BitisError : uint8_t { Ok = 0, UnexpectedEnd, InvalidValue,
  InvalidDiscriminant, TrailingData, ValueOutOfRange };`
- Sticky error state in `BitisDeserializer` / `BitisSerializer`; reads past the
  end return 0 and set `UnexpectedEnd`.
- New: `try_deserialize<T>(const uint8_t*, size_t, T&, BitisSize* = nullptr)`,
  vector overload, `try_deserialize_exact<T>`, `try_serialize<T>(const T&,
  std::vector<uint8_t>&, BitisSize* = nullptr)`.
- Legacy `serialize`, `deserialize`, `deserialize_with_size` keep working.
- Must stay usable on ESP32 firmware: C++17, no exceptions, no RTTI.

## Compiler

- Deterministic output, `//|` doc comment fix, validation pass with
  file:line:col errors, schema stamp.
- Schema stamp: generated code exposes the compiler version and a 64-bit schema
  hash (first 8 bytes of SHA-256 over the canonical schema description, lower-
  case hex, 16 digits). Template variables: `bitis_version` (string) and
  `schema_hash` (16 hex digits, no prefix).
  - Rust: `pub const BITIS_VERSION: &str`, `pub const BITIS_SCHEMA_HASH: u64`
  - C++: `constexpr const char* BITIS_VERSION`, `constexpr uint64_t BITIS_SCHEMA_HASH`
  - Dart: `const String bitisVersion`, `final Int64 bitisSchemaHash` (+ hex string)

## Dart

- `dart/bitis/`: pure-Dart runtime package `bitis` (web-safe).
- Compiler target `-l dart` generating classes per message/enum/oneof
  (`render_dart`, `src/lib_impl/dart.rs`, `templates/*.dart.jinja`):
  - one library per invocation, imports only `package:bitis/bitis.dart` and
    `dart:typed_data`; no Dart toolchain needed to generate,
  - messages: immutable `final class X implements BitisMessage` (final fields,
    const constructor where all defaults are constant, `copyWith`), defaults =
    Rust `Default` (fixed point: overflow),
  - enums: Dart enums (`bitisName` = schema name, `defaultValue`); oneofs:
    `sealed class OO_{Msg}_{Oneof}` + `OO_{Msg}_{Oneof}_{Element}` subclasses,
  - JSON mapping as in `BitisJson`; fixed point equality on the raw value,
  - laid out like `dart format` (tall style, width 80) and marked
    `// dart format off`,
  - tests: `tests/codegen_snapshots.rs`, `dart/codegen_test/` (cross-check
    against the Rust runtime with `fixtures/rust_vectors.json`).
- Type mapping: `int` for widths ≤ 32 bits, `Int64` (package:fixnum) for
  33–64 bits signed and unsigned ≤ 63 bits, `BigInt` for unsigned 64-bit.

## Golden vectors

- `golden/`: one schema covering all types, value cases, committed expected
  bytes (produced by Rust), plus invalid inputs with the expected error kind.
  Rust, C++ and Dart tests must all pass against the same files.
  Format, regeneration and how each runtime runs them: golden/README.md
  (`golden/run_all.sh` runs the Rust tests, the C++ tests plain and with
  ASan/UBSan, the Python tests if a venv with maturin is available, and the
  Dart tests on the VM and in Chrome with dart2js and dart2wasm).
- Dart: package `golden/dart/` tests the runtime with the generated
  `golden/gen/dart/golden.dart` (up-to-date check in tests/golden.rs); the
  manifest is embedded as `test/manifest.g.dart` (`tool/embed_manifest.dart`,
  staleness checked on the VM) so that the same tests run in the browser.
  Same checks and spot checks as Rust/C++, plus a JSON round trip of every
  valid case.

## Release block (2026-10-20, ordered by the user)

API policy for the rest of 0.11: still no wire format change. The C++ footprint
work may change C++ *data members* (fixed point storage, name members);
member functions keep their names and meaning. Everything else stays additive.

### C++ footprint (cpp_lib/bitis_lib.h, templates/data_objects.cpp.jinja)
- `BITIS_NO_PRINT`: if defined before including the generated header, every
  `print()` stays callable but is an empty inline function, and no name strings
  (attribute, enum value, type names) end up in the binary.
- Attribute names: one `static constexpr` table per message type instead of
  per-instance pointer members (0 bytes per object).
- Thin templates over shared non-template core functions (fixed point
  conversion, dyn integers, integers): the per-type code only passes constants.
- Caller-provided buffer: `try_serialize(const T&, uint8_t *buf, std::size_t cap,
  BitisSize* = nullptr)`; new `BitisError::BufferTooSmall` (C++ only).
  `bitis_max_bits<T>()` / `bitis_max_bytes<T>()` (constexpr, `BITIS_UNBOUNDED`
  if unbounded) to size such buffers at compile time.
- Fixed point stores the raw value (smallest unsigned type holding N bits)
  instead of `double` + state; `value()`, state accessors and `set()` compute it.
  Equality on the raw value (spec 2.7). Default raw value = overflow.
- Every bitis C++ type provides `static constexpr std::size_t min_bits()` and
  `static constexpr std::size_t max_bits()` (spec section 4).
- `print()` of all new types must be wrapped in `#ifndef BITIS_NO_PRINT`
  (with an empty `#else` variant).

Done (branch wt/cppfoot), decisions:
- Shared core: `namespace bitis_core` (`write_uint`/`read_uint`, `write_dyn`/
  `read_dyn`, `read_enum`, `write_bytes`/`read_bytes`, `fp_to_raw`/`fp_from_raw`),
  marked `BITIS_NOINLINE` (GCC/Clang `noinline`, overridable) like
  `write_bits`/`read_bits`; the gain of `noinline` itself is small (it keeps
  one copy per program).
- Messages and oneofs use one fold expression per type and operation instead of
  a recursive template chain (large gain at `-Og`). Nested messages, oneofs and
  list elements are decoded in place (`bitis_helper::read_into`, containers get
  a `read_into()` member) instead of being moved through temporaries; the bit
  count of `deserialize()` is the consumed difference (identical on success).
- One bit writer: `BitisSerializer` writes into `data_cache` or a caller buffer
  (`out_buf`/`out_cap`); on `BufferTooSmall` the bit count continues, so `size`
  reports the needed size.
- Fixed point: new accessors `value()`, `state()`, `raw()`, `set_raw()`; the
  old ones (`get_value()`, `get_state()`, `set()`, `val_to_u64`/`u64_to_val`)
  are unchanged. The removed members were private already.
- Not done: smaller `BitisEnum` storage (firmware prints `enum.value.value`
  with `PRIu32`, so `value` stays `DynInteger<uint32_t, ...>`) and a smaller
  oneof selector (moving it behind the union gains nothing, data members do not
  reuse tail padding; would need the selector inside `UnionT`).
- Validation: `min_bits` and `max_bits` are reserved C++ member names now;
  `str_<attr>` is no longer reserved (the members are gone).

### Maximum lengths `[M]` (spec 2.16)
- Schema syntax `astr_dY[M]`, `binary_dY[M]`, `repeated_dyn_Y[M] T`,
  `1 ≤ M ≤ 65535`; part of the canonical schema / hash.
- All runtimes enforce the bound (decode `InvalidValue`, encode
  `ValueOutOfRange`, legacy encode truncates to `M`).
- C++: bounded fields use heap-free containers (`std::array` + size); unbounded
  fields keep `std::vector`. Rust/Dart/Python keep their container types.
- `max_bits` (spec section 4) in Rust (`Option<u64>`), C++ and Dart; golden
  manifest gets `max_bits` lines.
- Status (branch wt/bounds): done. Rust `BoundedAString<Y, M>`, `BoundedBinary<Y, M>`,
  `BoundedDynArray<T, Y, M>` and `max_bits()` for all types (derive macros for messages,
  enums, oneofs); C++ `BoundedAString/BoundedBinary/BoundedDynArray` (one block in
  bitis_lib.h); Dart `maxLength` parameters, `MaxBits`, generated `bitisMaxBits`; Python
  checked setters (`ValueError`). Golden messages `BoundsItem`/`Bounds` and `max_bits`
  lines; the C++ golden test skips `max_bits` until every C++ type has `max_bits()`.

### Python (bindings generated via pyo3)
- `.pyi` stub fixed (types of optionals/strings, `serialize()` result),
  exception types `DecodeError` / `EncodeError` (subclasses of `ValueError`,
  attribute `kind` = spec error name), wheel version = bitis version, abi3.
- Python golden tests (maturin in a venv).

Done (branch wt/python):
- pyo3 0.29: the generated code uses the current pyo3 API (`Python::attach`,
  `#[pyclass(from_py_object)]`, no `PyObject`), it did not compile with the
  pyo3 0.29 that `maturin init` selects. The compiler sets in the Cargo.toml of
  the maturin project: `package.version` = bitis version, `pyo3 = { version =
  "0.29", features = ["abi3-py38"] }` (other features and an existing `abi3*`
  feature are kept), `strum_macros = "0.27"` if missing, `bitis` = compiler
  version unless `--prevent-update-bitis-lib-in-crate` (src/lib_impl/python.rs).
  Wheel: `<name>-<bitis version>-cp38-abi3-<platform>.whl`, CPython >= 3.8.
- Module exceptions `DecodeError` / `EncodeError` (`ValueError` subclasses,
  attribute `kind`, message `"<kind>: <description>"`); Rust helpers
  `decode_error(kind, detail)` / `encode_error(kind, detail)` in pyrust.rs;
  bounded setters / constructors raise `EncodeError`.
- Per message (additive): `try_serialize()` (same tuple as `serialize()`,
  raises `EncodeError` incl. for non-ASCII strings), `try_deserialize(data) ->
  (msg, total_bits, total_bytes)`, `try_deserialize_exact(data)`, `min_bits()`,
  `max_bits()`. `deserialize()` raises `DecodeError` instead of `Exception`.
  These names are reserved as attribute names.
- Enums `eq, eq_int, hash` (decoded values compare equal and are hashable);
  oneof selector enums hashable.
- `.pyi`: enum / oneof / selector classes, real attribute names and types
  (`int | None`, `str`, `bytes`, `list[T]`), fixed point Overflow/Underflow
  read-back values as comments, `__new__` signatures, `@final`; checked by
  `mypy --strict` and `mypy.stubtest` in the golden tests.
- Fix: binary element inside a oneof did not compile (Display).
- golden/python (build.sh, test_golden.py), run_all.sh python step; build.sh
  also cargo-checks the Python bindings of every codegen snapshot.

### CRC-32 helper (requested by fe-agent and fw-agent)
- CRC-32/ISO-HDLC (reflected poly 0xEDB88320, init and xorout 0xFFFFFFFF, same as
  zlib.crc32 and the ESP32 ROM call `esp_rom_crc32_le(0, buf, len)` — the ROM
  function inverts on entry and exit itself; continue with `esp_rom_crc32_le(c, ...)`),
  check value `crc32("123456789") = 0xCBF43926`.
- API in every runtime: one-shot `crc32(bytes) -> u32` and an incremental
  `Crc32` (update / finish). Not part of the wire format, no envelope message.
- Implemented: Rust `bitis_lib::crc32` / `Crc32` (256-entry const table), C++
  `cpp_lib/bitis_crc32.h` with the runtime's prefix: `bitis_crc32()` / `BitisCrc32`
  (avoids zlib's global `crc32`; 16-entry nibble table, 64 bytes, by default,
  `BITIS_CRC32_TABLE256` for 1 KiB), Dart `crc32` / `Crc32` (`value` and `finish()`),
  Python `crc32` / `Crc32` in every generated module. ESP32: `esp_rom_crc32_le(0, buf, len)`
  gives the same value (it inverts on entry/exit itself; not `~esp_rom_crc32_le(~0, ...)`).
  Vectors: `golden/vectors/crc32.txt` (generated with Python `zlib.crc32`).
