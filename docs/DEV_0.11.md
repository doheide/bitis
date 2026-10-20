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
  ASan/UBSan, and the Dart tests on the VM and in Chrome with dart2js and
  dart2wasm; Python is reported as SKIP).
- Dart: package `golden/dart/` tests the runtime with the generated
  `golden/gen/dart/golden.dart` (up-to-date check in tests/golden.rs); the
  manifest is embedded as `test/manifest.g.dart` (`tool/embed_manifest.dart`,
  staleness checked on the VM) so that the same tests run in the browser.
  Same checks and spot checks as Rust/C++, plus a JSON round trip of every
  valid case.
