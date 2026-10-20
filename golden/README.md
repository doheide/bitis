# bitis golden vectors

Language-neutral test vectors for the bitis wire format (docs/WIRE_FORMAT.md). Every
runtime (Rust, C++, Python via Rust, Dart) tests against the SAME committed files, so a
difference between a runtime and the spec, or between two runtimes, is detected.

The Rust runtime is the reference: valid vectors are the bytes it produces. Invalid and
non-canonical inputs are built bit by bit by the generator with an independent writer (a
direct transcription of the spec) together with the error the SPEC requires; the generator
refuses to write the manifest if the Rust runtime disagrees.

## Layout

```
golden/
  schema/golden.bitis       one schema covering every type and edge
  gen/rust/golden.rs        generated from the schema (committed, see "Regeneration")
  gen/cpp/golden.h          generated from the schema (includes "bitis_lib.h" from cpp_lib/)
  vectors/manifest.txt      the vectors (GENERATED, do not edit)
  gen_rust/                 generator of manifest.txt (cargo project, path dependency on bitis)
  cpp/golden_test.cpp       C++ test (gtest target bitis_golden, built by cpp_lib/tests)
  run_all.sh                runs all golden tests
tests/golden.rs             Rust test (cargo test --test golden)
```

## Schema

`schema/golden.bitis` contains (message types used in the manifest in bold):

| Message | Covers |
|---|---|
| **Ints** | bool; uint_1/7/32/33/63/64; int_1/7/16/32/33/64 |
| **DynInts** | uint_8d3, uint_32d4, uint_64d64, uint_64d7, int_16d5, int_64d8 |
| **Floats** | float, double |
| **FixedPoints** | fp_2[0,1], fp_10[-2,3], fp_34[-90,90] (GNSS lat), fp_35[-180,180] (GNSS lng), fp_53[-1000000,1000000] |
| **Texts** | astr_d4, binary_d4 |
| **Leaf** / **Mid** / **Nested** | three nesting levels, optional message |
| **Containers** | optional scalar/message/string/enum; repeated_fixed of scalars and messages; repeated_dyn of scalars, bools, strings and messages |
| **Enums** | enum with 6 variants and dyn bits 2 (`Color`), enum with dyn bits 1 (`Level`), repeated enum |
| **IntList** | repeated_dyn of int_7 (the "repeated" oneof variant) |
| **Choice** | oneof(2) with bool / double / astr / message / repeated (IntList) / empty message / int_32d7 variants, followed by a bool |
| **Empty** | empty message (0 bits) |
| **Everything** | all of the above in one message |

## Manifest format

`vectors/manifest.txt` is line based so that it can be parsed without a JSON library.
Fields are separated by exactly one space; lines starting with `#` and empty lines are
comments. Hex strings are lower case without separators; `-` stands for empty input.

```
valid    <case> <MsgType> <total_bits> <hex>
noncanon <case> <MsgType> <total_bits> <input_hex> <canonical_hex>
invalid  <case> <MsgType> <error> <default|exact> <hex>
min_bits <MsgType> <min_bits>
```

`<case>` matches `[a-z0-9_]+` and is unique; `<MsgType>` is the message name from the
schema; `<error>` is one of `UnexpectedEnd`, `InvalidValue`, `InvalidDiscriminant`,
`TrailingData` (spec section 3). The header comment contains the schema hash of the schema
the vectors were generated for.

What every runtime has to check:

| Record | Check |
|---|---|
| `valid` | decode with the default and the exact decoder succeeds and consumes `total_bits`; re-encoding the decoded value gives exactly `hex`; with two extra bytes appended the default decoder still succeeds (same `total_bits`, same re-encoding) and the exact decoder fails with `TrailingData` |
| `noncanon` | decoding `input_hex` (default and exact) succeeds and consumes `total_bits`; re-encoding gives `canonical_hex` |
| `invalid` | decoding `hex` with the given decoder (`default`: `try_deserialize`, `exact`: `try_deserialize_exact`) fails with `error` |
| `min_bits` | `min_bits()` of the message type (spec section 4) |

Notes on the vectors:

- Floats only use the canonical quiet NaN (`0x7fc00000` / `0x7ff8000000000000`), see spec 2.6.
- Fixed point: the default value of a fixed point attribute is **Overflow** (raw `2^N-1`) in
  Rust and C++ (`fixedpoints_default`, `everything_default`). `fixedpoints_ties` contains
  values for which the encoder computes an exact tie `t = k + 0.5` with even `k`
  (round half away from zero gives `k + 1`, round half to even would give `k`);
  `fixedpoints_gnss` contains `fp2 = 0.49999999999999994`, which becomes the tie `1.5` in
  f64 arithmetic. These two are also checked as encoder spot checks (value -> bytes).
- Negative zero (sign bit 1, magnitude 0) is decode-only (`noncanon_*_negzero`).
- Decoders accept non-minimal dynamic integers (continuation bit 1 followed by zero chunks)
  for values, counts, enum and oneof indices, and ignore padding bits (`noncanon_*`).
- First error wins: an error is reported where it is detected, before the (missing) rest
  of the input (`enum_invalid_then_end`, `int16_invalid_then_end`); the count check of
  astr/binary/repeated_dyn comes before reading the elements (`astr_non_ascii_truncated`
  is `UnexpectedEnd`).

Cases (51 valid, 10 non-canonical, 65 invalid, 14 min_bits): defaults; min/max and 2^k
values of every integer width; i64/i32/i16 range limits (the base type minimum is invalid);
dyn int chunk boundaries (incl. uint_64d64 single chunk and the final 1 bit chunk of
uint_64d7); ±0.0, ±inf, NaN, subnormals, extremes; fixed point min/max/underflow/overflow/
NaN/±inf/ties/GNSS values; empty, ASCII edge (NUL, DEL) and long (multi chunk count)
strings and binaries; empty and full containers; every enum and oneof variant; deep
nesting; everything set; truncations of many vectors (incl. empty input); huge counts
(must fail without allocating); unknown enum/oneof indices (incl. 2^32-1, inside arrays,
optionals and nested messages); out-of-range magnitudes of int_16/int_32/int_64,
int_16d5/int_64d8/int_32d7; non-ASCII bytes (top level, nested, in arrays); trailing data
in exact mode.

## Running

All at once (Rust, C++ plain and with ASan/UBSan; Python and Dart are reported as SKIP):

```
golden/run_all.sh            # --no-asan to skip the sanitizer build
# offline: CARGO_FLAGS=--offline FETCHCONTENT_SOURCE_DIR_GOOGLETEST=<googletest checkout> golden/run_all.sh
```

Individually:

- **Manifest up to date**: `cargo run --manifest-path golden/gen_rust/Cargo.toml -- --check`
- **Rust**: `cargo test --test golden` (also checks that golden/gen/ is the current
  compiler output and that the manifest belongs to the schema)
- **C++**: target `bitis_golden` of cpp_lib/tests:
  ```
  cmake -S cpp_lib -B build && cmake --build build --target bitis_golden && build/tests/bitis_golden
  cmake -S cpp_lib -B build-asan -DBITIS_SANITIZE=ON && cmake --build build-asan --target bitis_golden && build-asan/tests/bitis_golden
  ```
  The manifest path is compiled in (`BITIS_GOLDEN_MANIFEST`). With ASan in some
  environments `ASAN_OPTIONS=verify_asan_link_order=0` is needed, also during the build
  (gtest_discover_tests runs the binary).
- **Python**: not covered. The Python bindings are generated Rust code built with maturin
  (`bitis compile -l python`), i.e. they use the Rust runtime that is tested here; a
  Python test would only exercise the pyo3 glue. To add one: build the golden schema as a
  maturin project, then for every manifest record call `<MsgType>.deserialize(bytes)` /
  `serialize()` and compare as above (maturin is not available in the CI environment of
  0.11).
- **Dart**: placeholder in run_all.sh, see below.

## Regeneration

After changing the schema, the compiler templates, or the runtime encoding:

```
cargo build
target/debug/bitis compile -l rust -i golden/schema/golden.bitis -o golden/gen/rust/golden.rs
target/debug/bitis compile -l cpp  -i golden/schema/golden.bitis -o golden/gen/cpp/golden.h --prevent-write-bitis-header-lib
cargo run --manifest-path golden/gen_rust/Cargo.toml      # rewrites vectors/manifest.txt
```

Review the diff of `vectors/manifest.txt`: changed bytes of existing cases mean the wire
format changed, which must not happen without a spec change. New cases are added in
`gen_rust/src/main.rs` (`valid`, `noncanon`, `invalid*`, `truncations`).

## Adding the Dart runtime (TODO, after the Dart codegen is merged)

1. Generate `golden/gen/dart/golden.dart` with `bitis compile -l dart` (add the command
   above and, like for Rust/C++, an up-to-date check).
2. Add a Dart test package (e.g. `golden/dart/`, depending on `dart/bitis` by path) whose
   test reads `../vectors/manifest.txt` (path relative to the package), splits lines on
   `' '`, maps `<MsgType>` to the generated class (a `switch` over the 14 names) and
   implements the checks of the table above with the Dart `tryDeserialize` /
   `tryDeserializeExact` / `trySerialize` equivalents. Map the error names to the Dart
   error enum by name.
3. Port the spot checks of tests/golden.rs (decoded values: ints at the i64 limits as
   `Int64`, `uint_64` as `BigInt`; fixed point values / Overflow / Underflow states;
   strings; enum and oneof variants; encoder: `fixedpoints_ties`, `fixedpoints_gnss`).
   Note: the default of a fixed point attribute must be Overflow as in Rust/C++
   (`fixedpoints_default`), and Dart must reproduce the f64 arithmetic of spec 2.7 exactly
   (also when compiled to JavaScript).
4. Replace the placeholder `dart_tests` in `run_all.sh` (`dart pub get && dart test`).

## Older cross-language framework (test_cases/)

`test_cases/t_*` exchange `.dat` files between the languages. Each `build.sh` now calls
the shared runner `test_cases/run_case.sh` (current compiler CLI, relative symlinks,
configurable via `BITIS`, `CARGO_FLAGS`, `CPP_BUILD_DIR`). All 8 cases pass for Rust and
C++, including decoding the committed `*.py.dat` files; the Python side is skipped unless
`test_cases/impl/python_impl/.venv` and maturin are available. Running a case rewrites the
generated `impl/*/messages.*` and the `.dat` files. The golden vectors supersede this
framework for wire format conformance (e.g. the GNSS fp_34/fp_35 types that are commented
out in t_simple_msg are covered here).
