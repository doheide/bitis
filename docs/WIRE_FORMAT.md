# bitis wire format (normative)

This document specifies the binary encoding produced and accepted by all bitis
runtimes (Rust, C++, Python via Rust, Dart). All runtimes MUST produce
byte-identical output for identical values and MUST accept/reject identical
inputs. The Rust runtime (`src/lib_impl/berde.rs`) is the reference
implementation; where this document and the reference disagree, it is a bug
in one of them and has to be resolved explicitly.

Version: applies to bitis 0.10.x and 0.11.x (the wire format did not change).

## 1. Bit stream

- A message is encoded into a single bit stream without any alignment or
  padding between fields.
- Bits are written LSB-first: stream bit `i` is bit `i % 8` (value `1 << (i % 8)`)
  of byte `i / 8`.
- A multi-bit value of width `w` is written least significant bit first, i.e.
  bit `k` of the value becomes stream bit `start + k`.
- After the last field the stream is padded with `0` bits up to the next byte
  boundary. The encoded size is `total_bits` and `total_bytes = ceil(total_bits / 8)`.
- Messages carry no header, tag, length prefix or version. The receiver must
  know the top-level message type.

## 2. Types

Notation: `W(v, w)` = write the low `w` bits of unsigned value `v` LSB-first.
`U32D(Y)` = the dynamic unsigned integer `uint_32dY` (section 2.4).

### 2.1 bool
`W(b ? 1 : 0, 1)` — 1 bit.

### 2.2 `uint_X` (1 ≤ X ≤ 64)
`W(v, X)`.
- Valid range: `0 ≤ v ≤ 2^X − 1`.

### 2.3 `int_X` (1 ≤ X ≤ 64) — sign-magnitude
`W(v < 0 ? 1 : 0, 1)` then `W(|v|, X)`. Total `X + 1` bits.
- Base type width `B`: 16 for `X ≤ 16`, 32 for `17 ≤ X ≤ 32`, 64 otherwise
  (this is the Rust/C++ storage type `i16`/`i32`/`i64`).
- Valid range: `|v| ≤ min(2^X − 1, 2^(B−1) − 1)`. In particular the minimum value of
  the base type (e.g. `i16::MIN`) is never valid. For `int_16`, `int_32`, `int_64`
  the range is therefore that of the base type without its minimum.
- Decoding: sign bit `1` with magnitude `0` ("negative zero") decodes to `0`.
  A magnitude outside the valid range is `InvalidValue`.

### 2.4 `uint_XdY` — dynamic unsigned integer (1 ≤ X ≤ 64, 1 ≤ Y ≤ X)
```
W(v != 0, 1)                       # flag
if v != 0:
    remaining = X
    loop:
        c = min(Y, remaining)
        W(v & (2^c − 1), c); v >>= c; remaining -= c
        if remaining == 0: break   # no continuation bit when X is used up
        W(v != 0, 1)               # continuation bit
        if v == 0: break
```
- Zero takes 1 bit. Example: `10` as `uint_32d3` → `1 010 1 100 0` (9 bits).
- Valid range: `0 ≤ v ≤ 2^X − 1` (and the base type range as in 2.3 for the
  storage type of width B: 16/32/64).
- Decoding accepts non-minimal encodings (continuation bit `1` followed by zero
  chunks). Decoded values outside the valid range are `InvalidValue`.
- Encoded size in bits: `1` for zero, otherwise `1 + Σ chunk bits + number of
  continuation bits written`.

### 2.5 `int_XdY` — dynamic signed integer
`W(v < 0, 1)` (always written, also for zero) followed by `uint_XdY` of `|v|`.
Zero takes 2 bits. Range and negative-zero handling as in 2.3 (with base type
width B derived from X).

### 2.6 `float`, `double`
IEEE 754 binary32 / binary64 bit pattern, `W(bits, 32)` / `W(bits, 64)`
(equivalent to the little-endian byte representation). Decoders accept any
NaN. Rust and C++ preserve NaN payloads on encode; the Dart runtime always
encodes the canonical quiet NaN (`0x7FC00000` / `0x7FF8000000000000`) because
JavaScript cannot preserve payloads. Test vectors only use the canonical NaN.

### 2.7 `fp_N[min, max]` — fixed precision (2 ≤ N ≤ 53, integer min < max)
`W(r, N)` with the raw value `r`:
- `r = 0` → Underflow, `r = 2^N − 1` → Overflow, otherwise a value.
- Let `S = (2^N − 3) as f64`, `R = (max as f64) − (min as f64)` (convert first, then
  subtract; `min`/`max` must lie within ±2^53).
- Encoding of a value `v` (all arithmetic in IEEE f64, in exactly this order):
  - `v > max` → Overflow; `v < min` → Underflow; `v` is NaN → Underflow
  - else `t = (v − min) / R * S + 1.0`, `r = round_half_away_from_zero(t)` (Rust `f64::round`)
- Decoding: `v = (r − 1) as f64 / S * R + min`.
- Runtimes expose Underflow/Overflow as explicit states, not as numeric
  sentinels.
- Re-encoding a decoded value is not guaranteed to reproduce the raw value for
  large N (f64 rounding, e.g. `fp_53[-1e6, 1e6]`); equality of fixed-point
  values is defined on the raw value.

### 2.8 `astr_dY`, `astr_dY[M]` — ASCII string
`U32D(Y)` byte count `n`, then `n` × `W(byte, 8)`.
- Every byte must be < 0x80. Decoding a byte ≥ 0x80 is `InvalidValue`;
  encoders must not be able to produce it.
- Optional maximum length `M` (bytes, see 2.16).

### 2.9 `binary_dY`, `binary_dY[M]`
`U32D(Y)` byte count `n`, then `n` × `W(byte, 8)`.
- Optional maximum length `M` (bytes, see 2.16).

### 2.10 `optional T`
`W(present, 1)`, then `T` if present.

### 2.11 `repeated_fixed_N T`
`N` × `T`, no prefix.

### 2.12 `repeated_dyn_Y T`, `repeated_dyn_Y[M] T`
`U32D(Y)` element count `n`, then `n` × `T`.
- Optional maximum number of elements `M` (see 2.16).

### 2.13 `enum E(Y)`
`U32D(Y)` of the zero-based variant index in declaration order.
An index ≥ number of variants is `InvalidDiscriminant`.

### 2.14 `oneof name(Y)`
`U32D(Y)` of the zero-based variant index in declaration order, then the
variant's payload. An index ≥ number of variants is `InvalidDiscriminant`.

### 2.15 `msg`
All attributes in declaration order, nothing else. An empty message encodes
to 0 bits.

### 2.16 Maximum length `[M]` (bitis 0.11)
`astr_dY[M]`, `binary_dY[M]` and `repeated_dyn_Y[M] T` declare a maximum
length `M` (bytes resp. elements), `1 ≤ M ≤ 65535`. The encoding is exactly the
same as without the bound (the count is still `U32D(Y)`); the bound only
restricts the valid values:
- Decoding: a count `n > M` is `InvalidValue`. This check is done directly after
  decoding the count, before the count check of section 3.
- Encoding: a length `> M` is `ValueOutOfRange`. Legacy (truncating) encoders
  write the first `M` bytes / elements instead.
- Runtimes may use storage without heap allocation for bounded fields (C++).
- The bound is part of the schema (and of the schema hash, section 5), not of
  the message.

## 3. Decoding rules (all runtimes)

Decoders operate on untrusted input and MUST NOT crash, read out of bounds,
loop forever or allocate memory proportional to a length field before the
corresponding data is known to be present.

- Reading past the end of the input is `UnexpectedEnd` (no implicit zero bits).
- For `astr`, `binary` and `repeated_dyn` with a maximum length `[M]`: a count
  `n > M` is `InvalidValue`, checked directly after decoding the count (section 2.16),
  i.e. before the following check.
- For `astr`, `binary` and `repeated_dyn`: after decoding the count `n`, if
  `n * max(1, min_bits(T)) > remaining_bits` the decoder fails with
  `UnexpectedEnd` before allocating anything (`min_bits(byte) = 8`). This rule
  never rejects valid data because element types with `min_bits = 0` (messages
  without any bits) are rejected by the compiler inside `repeated_dyn`.
- Unknown enum/oneof index → `InvalidDiscriminant`; out-of-range value or
  non-ASCII string byte → `InvalidValue`.
- Trailing bytes after the message are ignored by the default decode function
  and reported as consumed size. The strict ("exact") decode function fails with
  `TrailingData` if the input is longer than `ceil(total_bits / 8)` bytes.
  Padding bits are not checked.

Error kinds (same names in every runtime):

| Kind | Meaning |
|---|---|
| `UnexpectedEnd` | input ended before the message was complete (in a stream: wait for more data) |
| `InvalidValue` | a field value is outside its valid range / non-ASCII string / count above a maximum length `[M]` |
| `InvalidDiscriminant` | unknown enum or oneof index |
| `TrailingData` | strict decode only: more bytes than the message needs |

Encoding errors:

| Kind | Meaning |
|---|---|
| `ValueOutOfRange` | an integer value does not fit its declared width / base type, a string is not ASCII, or a length exceeds its maximum length `[M]` |

## 4. min_bits, max_bits

`min_bits(T)` is a lower bound of the encoded size of `T` (exact for fixed-size
types) and is identical in all runtimes:
bool 1; `uint_X` X; `int_X` X+1; `uint_XdY` 1; `int_XdY` 2; float 32; double 64;
`fp_N` N; astr/binary/repeated_dyn 1 (count zero); optional 1;
`repeated_fixed_N T` N·min_bits(T); enum 1;
oneof `1 + min over variants of min_bits(variant)`; msg Σ attributes.

`max_bits(T)` is an upper bound of the size of the encoding *produced by an
encoder* for any valid value of `T`, or *unbounded*, computed exactly by the
rules below (it can exceed the real maximum by a few bits for signed dyn
integers at the base type limit). It is identical in all runtimes (Rust:
`Option<u64>` with `None` = unbounded; C++: `std::size_t` with
`BITIS_UNBOUNDED`, saturating arithmetic). Decoders accept non-minimal dyn
integers, so accepted inputs may be longer.
- `dyn(v, X, Y)` = size of the `uint_XdY` encoding of `v` (section 2.4);
  `dynmax(X, Y)` = `dyn(2^X − 1, X, Y)` = `1 + X + ceil(X / Y) − 1`.
- bool 1; `uint_X` X; `int_X` X+1; `uint_XdY` `dynmax(X, Y)`;
  `int_XdY` `1 + dynmax(X, Y)`; float 32; double 64; `fp_N` N;
  `astr_dY[M]` / `binary_dY[M]` `dyn(M, 32, Y) + 8·M`;
  `repeated_dyn_Y[M] T` `dyn(M, 32, Y) + M·max_bits(T)`;
  astr/binary/repeated_dyn without `[M]` unbounded;
  optional `1 + max_bits(T)`; `repeated_fixed_N T` N·max_bits(T);
  enum with `k` values `dyn(k − 1, 32, Y)`;
  oneof with `k` variants `dyn(k − 1, 32, Y) + max over variants of max_bits(variant)`;
  msg Σ attributes. Anything containing an unbounded part is unbounded.
- A result that the runtime cannot represent is reported as unbounded: Rust above
  `u64::MAX` (`None`), C++ above `SIZE_MAX` (saturates to `BITIS_UNBOUNDED`), Dart above
  `2^53 − 1` (`null`, not representable on the web). Realistic schemas stay far below.

## 5. Schema hash

The schema hash is not part of the encoding (messages carry no header). Generated
code exposes it (with the compiler version) so that peers can compare schemas
out of band, e.g. in a handshake.

`schema_hash = first 8 bytes of SHA-256(canonical schema)`, read as a big-endian
unsigned 64-bit integer; as text it is written as 16 lower-case hex digits
without prefix (i.e. the first 16 hex digits of the SHA-256 hex digest).

The canonical schema is ASCII text (implemented by `canonical_schema()` in
`src/lib_impl/compiler.rs`, reusable by every target):

```
bitis-schema-v1\n
<one line per enum / message, sorted by name (byte order), each ending with \n>
```

- enum: `enum NAME(Y){V1,*V2,V3}` — values in declaration order, the default
  value prefixed with `*`, `Y` the dyn bit size.
- message: `msg NAME{ATTR;ATTR;}` or, with a parent, `msg NAME:PARENT{ATTR;}`;
  attributes in declaration order, each followed by `;` (`msg NAME{}` if empty).
- attribute: `[MOD ]TYPE NAME` with `MOD` one of `optional`, `repeated_fixed_N`,
  `repeated_dyn_Y`.
- oneof attribute: `oneof NAME(Y){[*]TYPE NAME;...}` — elements in declaration
  order, the default element prefixed with `*` (nested oneofs recursively).
- TYPE: `bool`, `uint_X`, `int_X`, `uint_XdY`, `int_XdY`, `float`, `double`,
  `fp_N[MIN,MAX]` (decimal integers, `-` for negative values, no `+`, no
  leading zeros), `astr_dY`, `binary_dY`, `astr_dY[M]`, `binary_dY[M]`, or the
  name of a message / enum.
- A bounded modifier is written `repeated_dyn_Y[M]`. Without a bound the text
  (and therefore the hash of existing schemas) is unchanged.
- Single spaces only where shown above; no other whitespace, no comments.

Example: `enum E(2) { *A, B }` and `msg M { optional uint_3 x; oneof o(2) { *bool b; E e; } }`
give

```
bitis-schema-v1
enum E(2){*A,B}
msg M{optional uint_3 x;oneof o(2){*bool b;E e;};}
```

with the schema hash `04ee18ca6b3375b1` (check: `printf 'bitis-schema-v1\nenum E(2){*A,B}\nmsg M{optional uint_3 x;oneof o(2){*bool b;E e;};}\n' | sha256sum`).

The hash therefore depends on all names, types, bit sizes, defaults and the
attribute / value order, but not on comments, formatting, the order of the
definitions or the split into input files.
