# bitis for Dart

`dart/bitis/` is the pure-Dart runtime package `bitis` for the bitis
bit-level serialization format (wire format: [`docs/WIRE_FORMAT.md`](../docs/WIRE_FORMAT.md)).
Code generated with `bitis -l dart` uses it. It runs on the Dart VM, Flutter
(Android, iOS, Linux, ...), and on the web with both dart2js and dart2wasm.

- Dart SDK `>=3.5.0 <4.0.0`, null safe.
- Only dependency: [`fixnum`](https://pub.dev/packages/fixnum) (`Int64`, re-exported).
- No `dart:io`, `dart:ffi` or `dart:mirrors`.
- The package is not published to pub.dev.

## Depending on it

From git (pin `ref` to a tag or commit):

```yaml
dependencies:
  bitis:
    git:
      url: https://github.com/doheide/bitis.git
      path: dart/bitis
      ref: v0.11.0
```

From a local checkout:

```yaml
dependencies:
  bitis:
    path: ../bitis/dart/bitis
```

Then `import 'package:bitis/bitis.dart';`.

## Type mapping

| bitis type | Dart type | writer / reader |
|---|---|---|
| `bool` | `bool` | `writeBool` / `readBool` |
| `uint_X`, X ≤ 32 | `int` | `writeUint(v, X)` / `readUint(X)` |
| `uint_X`, 33 ≤ X ≤ 63 | `Int64` | `writeUintInt64` / `readUintInt64` |
| `uint_64` | `BigInt` | `writeUintBigInt` / `readUintBigInt` |
| `int_X`, X ≤ 32 | `int` | `writeInt(v, X)` / `readInt(X)` |
| `int_X`, 33 ≤ X ≤ 64 | `Int64` | `writeIntInt64` / `readIntInt64` |
| `uint_XdY` / `int_XdY` | as above | `writeDynUint(v, X, Y)`, `...Int64`, `...BigInt`, `writeDynInt`, `writeDynIntInt64` (+ `read...`) |
| `float` / `double` | `double` | `writeFloat32` / `writeFloat64` |
| `fp_N[min, max]` | `FixedPoint` | `writeFixedPoint(v, const FixedPointType(N, min, max))` |
| `astr_dY` | `String` (ASCII) | `writeAStr(v, Y)` / `readAStr(Y)` |
| `binary_dY` | `Uint8List` | `writeBinary(v, Y)` / `readBinary(Y)` |
| `optional T` | `T?` | `writeOptional(v, fn)` / `readOptional(fn)` |
| `repeated_fixed_N T` | `List<T>` | `writeFixedArray(v, N, fn)` / `readFixedArray(N, fn)` |
| `repeated_dyn_Y T` | `List<T>` | `writeDynArray(v, Y, fn)` / `readDynArray(Y, minBits(T), fn)` |
| `enum E(Y)` | Dart enum | `writeEnumIndex(i, count, Y)` / `readEnumIndex(count, Y)` |
| `oneof name(Y)` | sealed class | `writeOneOfIndex(i, count, Y)` / `readOneOfIndex(count, Y)` + payload |
| `msg` | class extending `BitisMessage` | `bitisWrite(w)` / `static T bitisRead(r)` |

`int_X` uses the base type range rule of the spec (2.3): `int_16` is
`-32767..32767`, `int_32` is `±(2^31 - 1)`, `int_64` excludes `Int64.MIN_VALUE`.

## Generated code

`bitis compile -l dart -o lib/messages.dart -i a.bitis -i b.bitis` generates
one library with a class per message, a Dart enum per enum and a sealed class
hierarchy per oneof (see "Dart" in the [top level README](../README.md)):

```dart
import 'package:bitis/bitis.dart';
import 'messages.dart';

final bytes = const DTS_Msg_Ping(dummy: 2).toBytes();
final ping = DTS_Msg_Ping.fromBytesExact(bytes);
final json = ping.toJson(); // {'dummy': 2}
```

Besides `BitWriter`/`BitReader`, generated code uses `BitisJson`, `BitisEq`
(equality / hash of lists and fixed point values) and `bitisKeep` /
`bitisKeepOr` (`copyWith` of optional attributes).

`dart/codegen_test/` tests the generated code (VM, dart2js, dart2wasm) for
`test.bitis`, the carachang protocol schemas and further fixtures, including
a cross-check against bytes produced by the Rust runtime; `tool/regen.sh`
there regenerates the code (`--vectors`: also the Rust vectors).

## Usage (hand-written messages)

```dart
import 'package:bitis/bitis.dart';

class Reading extends BitisMessage {
  Reading(this.id, this.temp, this.name);
  int id; // uint_12
  FixedPoint temp; // fp_10[-40, 125]
  String? name; // optional astr_d4

  static const _tempType = FixedPointType(10, -40, 125);

  @override
  void bitisWrite(BitWriter w) {
    w.writeUint(id, 12);
    w.writeFixedPoint(temp, _tempType);
    w.writeOptional(name, (w, v) => w.writeAStr(v, 4));
  }

  static Reading bitisRead(BitReader r) => Reading(
        r.readUint(12),
        r.readFixedPoint(_tempType),
        r.readOptional((r) => r.readAStr(4)),
      );
}

final bytes = bitisEncode(Reading(7, const FixedPoint.value(21.5), null));
final decoded = bitisDecode(bytes, Reading.bitisRead); // .value, .totalBits, .totalBytes
final strict = bitisDecodeExact(bytes, Reading.bitisRead); // TrailingData check
```

Errors: decoding throws `BitisDecodeException` (`kind`: `unexpectedEnd`,
`invalidValue`, `invalidDiscriminant`, `trailingData`), encoding throws
`BitisEncodeException` (`valueOutOfRange`). Decoding untrusted input never
throws anything else, never reads out of bounds and never allocates memory
proportional to an unchecked length field.

JSON helpers for generated `toJson`/`fromJson` are in `BitisJson`;
`min_bits` per spec section 4 is in `MinBits`.

## Web safety

Under dart2js an `int` is a double (exact up to 2^53) and bitwise operators
are 32-bit. The runtime therefore handles every value of up to 64 bits as two
32-bit halves, applies bitwise operators only to values below 2^32 with shift
distances below 32, combines halves with exact arithmetic, and converts floats
with `ByteData.setFloat32/getFloat32/setFloat64/getFloat64` byte by byte
(`getUint64`/`getInt64` are never used). NaN is always written as the canonical
quiet NaN (`0x7FC00000` / `0x7FF8000000000000`) so that every Dart platform
produces identical bytes.

## Development

Runtime:

```sh
cd dart/bitis
dart pub get
dart analyze
dart format .
dart test -p vm,chrome                    # CHROME_EXECUTABLE=/path/to/chrome if needed
dart test -p chrome --compiler dart2wasm
```

Generated code (`dart/codegen_test`):

```sh
cd dart/codegen_test
dart pub get
tool/regen.sh            # regenerate lib/*.dart from fixtures/schemas (needs cargo)
tool/regen.sh --vectors  # also fixtures/rust_vectors.json via tool/rust_vectors
dart analyze
dart test -p vm,chrome
dart test -p chrome --compiler dart2wasm
```

`test/rust_vectors.dart` holds byte vectors produced by the Rust reference
runtime (valid values only). Regenerate with the program in
`tool/rust_vectors/` (see its `Cargo.toml`).

Cross-language golden vectors (`golden/dart`: the same manifest as the Rust
and C++ runtimes, see [`golden/README.md`](../golden/README.md)):

```sh
cd golden/dart
dart pub get
dart test -p vm,chrome
dart test -p chrome --compiler dart2wasm
```
