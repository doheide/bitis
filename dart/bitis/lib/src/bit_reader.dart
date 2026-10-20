import 'dart:typed_data';

import 'package:fixnum/fixnum.dart';

import 'errors.dart';
import 'fixed_point.dart';
import 'int_util.dart';

/// Callback reading one element / optional payload / message. Generated code
/// passes tear-offs like `Foo.bitisRead` or closures like
/// `(r) => r.readUint(5)`.
typedef BitisReadFn<T> = T Function(BitReader r);

/// Reads a bitis bit stream (see `docs/WIRE_FORMAT.md`).
///
/// Every `read*` method either returns a valid value or throws
/// [BitisDecodeException]:
/// * [BitisDecodeErrorKind.unexpectedEnd] if the input ends early (nothing
///   past the end is ever read, no implicit zero bits),
/// * [BitisDecodeErrorKind.invalidValue] for out-of-range values or non-ASCII
///   strings,
/// * [BitisDecodeErrorKind.invalidDiscriminant] for unknown enum/oneof
///   indices.
///
/// After an exception the reader position is unspecified. Decoding never
/// allocates memory proportional to an unchecked length field and every loop
/// is bounded by the input size or the (trusted) type parameters.
/// Invalid type parameters (bit widths outside the documented range) are
/// programming errors and throw [ArgumentError].
final class BitReader {
  /// Creates a reader over [data]. A [Uint8List] is used without copying
  /// (it must not be modified while reading); any other list is copied (its
  /// elements are truncated to 8 bits).
  BitReader(List<int> data)
      : _data = data is Uint8List ? data : Uint8List.fromList(data);

  final Uint8List _data;
  int _byte = 0; // index of the current byte
  int _bit = 0; // 0..7, bit offset inside the current byte
  final ByteData _scratch = ByteData(8);

  /// The input bytes.
  Uint8List get data => _data;

  /// Number of input bits (`8 * data.length`).
  int get totalBits => _data.length * 8;

  /// Number of bits consumed so far.
  int get bitPosition => _byte * 8 + _bit;

  /// Number of input bits not consumed yet.
  int get remainingBits => totalBits - bitPosition;

  /// Number of bytes touched so far: `ceil(bitPosition / 8)`.
  int get bytePosition => _byte + (_bit > 0 ? 1 : 0);

  Never _fail(BitisDecodeErrorKind kind, String message) =>
      throw BitisDecodeException(kind, message, bitPosition);

  void _need(int bits) {
    if (bits > remainingBits) {
      _fail(
        BitisDecodeErrorKind.unexpectedEnd,
        'need $bits bits, only $remainingBits left',
      );
    }
  }

  /// Reads [bits] (0..32) bits; the caller has checked availability.
  int _take(int bits) {
    var result = 0;
    var shift = 0;
    while (bits > 0) {
      final avail = 8 - _bit;
      final take = bits < avail ? bits : avail;
      final chunk = (_data[_byte] >>> _bit) & lowMask[take];
      result += chunk * pow2[shift];
      shift += take;
      bits -= take;
      _bit += take;
      if (_bit == 8) {
        _bit = 0;
        _byte++;
      }
    }
    return result;
  }

  /// Reads [count] bits and places them at bit [shift] of the 64-bit
  /// accumulator `(lo, hi)`. Requires `shift + count <= 64`.
  (int, int) _takeSlice(int lo, int hi, int shift, int count) {
    _need(count);
    while (count > 0) {
      int n;
      if (shift < 32) {
        n = 32 - shift;
        if (n > count) n = count;
        lo += _take(n) * pow2[shift];
      } else {
        n = 64 - shift;
        if (n > count) n = count;
        hi += _take(n) * pow2[shift - 32];
      }
      shift += n;
      count -= n;
    }
    return (lo, hi);
  }

  /// Dynamic unsigned integer body (spec 2.4). Returns the value as
  /// `(lo, hi)`; it always has at most [maxBits] bits. At most
  /// `maxBits + maxBits / chunkBits + 1` bits are read.
  (int, int) _takeDyn(int maxBits, int chunkBits) {
    var lo = 0;
    var hi = 0;
    if (!readBool()) return (0, 0);
    var shift = 0;
    var remaining = maxBits;
    while (true) {
      final c = chunkBits < remaining ? chunkBits : remaining;
      (lo, hi) = _takeSlice(lo, hi, shift, c);
      shift += c;
      remaining -= c;
      if (remaining == 0) break;
      if (!readBool()) break;
    }
    return (lo, hi);
  }

  static void _checkBits(int bits, int max) {
    if (bits < 1 || bits > max) {
      throw ArgumentError.value(bits, 'bits', 'must be in 1..$max');
    }
  }

  static void _checkDyn(int maxBits, int chunkBits, int max) {
    _checkBits(maxBits, max);
    if (chunkBits < 1 || chunkBits > maxBits) {
      throw ArgumentError.value(
          chunkBits, 'chunkBits', 'must be in 1..$maxBits');
    }
  }

  /// Validates a signed magnitude (spec 2.3) and returns it as `(lo, hi)`.
  (int, int) _checkMagnitude(int lo, int hi, int bits, String type) {
    if (bitLengthU64(lo, hi) > signedMagnitudeBits(bits)) {
      _fail(
          BitisDecodeErrorKind.invalidValue, 'magnitude out of range of $type');
    }
    return (lo, hi);
  }

  int _signedInt(bool neg, int lo, int hi, int bits, String type) {
    _checkMagnitude(lo, hi, bits, type);
    final mag = joinInt(lo, hi);
    return (neg && mag != 0) ? -mag : mag; // negative zero decodes to 0
  }

  Int64 _signedInt64(bool neg, int lo, int hi, int bits, String type) {
    _checkMagnitude(lo, hi, bits, type);
    final mag = joinInt64(lo, hi);
    return neg ? -mag : mag; // -Int64.ZERO == Int64.ZERO
  }

  // ---------------------------------------------------------------- basics

  /// Reads [bits] (1..32) raw bits as an unsigned value. Same as [readUint].
  int readBits(int bits) => readUint(bits);

  /// `bool` (spec 2.1).
  bool readBool() {
    _need(1);
    return _take(1) != 0;
  }

  // ------------------------------------------------------- fixed integers

  /// `uint_X` with `X = bits <= 32` (spec 2.2).
  int readUint(int bits) {
    _checkBits(bits, 32);
    _need(bits);
    return _take(bits);
  }

  /// `uint_X` with `X = bits <= 63` as [Int64] (spec 2.2).
  Int64 readUintInt64(int bits) {
    _checkBits(bits, 63);
    final (lo, hi) = _takeSlice(0, 0, 0, bits);
    return joinInt64(lo, hi);
  }

  /// `uint_X` with `X = bits <= 64` as [BigInt] (spec 2.2).
  BigInt readUintBigInt(int bits) {
    _checkBits(bits, 64);
    final (lo, hi) = _takeSlice(0, 0, 0, bits);
    return joinBigInt(lo, hi);
  }

  /// `int_X` with `X = bits <= 32` (spec 2.3). A magnitude above
  /// `min(2^X - 1, 2^(B-1) - 1)` is [BitisDecodeErrorKind.invalidValue];
  /// negative zero decodes to 0.
  int readInt(int bits) {
    _checkBits(bits, 32);
    _need(bits + 1);
    final neg = _take(1) != 0;
    final (lo, hi) = _takeSlice(0, 0, 0, bits);
    return _signedInt(neg, lo, hi, bits, 'int_$bits');
  }

  /// `int_X` with `X = bits <= 64` as [Int64] (spec 2.3).
  Int64 readIntInt64(int bits) {
    _checkBits(bits, 64);
    _need(bits + 1);
    final neg = _take(1) != 0;
    final (lo, hi) = _takeSlice(0, 0, 0, bits);
    return _signedInt64(neg, lo, hi, bits, 'int_$bits');
  }

  // ----------------------------------------------------- dynamic integers

  /// `uint_XdY` with `X = maxBits <= 32`, `Y = chunkBits` (spec 2.4).
  /// Non-minimal encodings are accepted.
  int readDynUint(int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits, 32);
    final (lo, hi) = _takeDyn(maxBits, chunkBits);
    return joinInt(lo, hi);
  }

  /// `uint_XdY` with `X = maxBits <= 63` as [Int64] (spec 2.4).
  Int64 readDynUintInt64(int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits, 63);
    final (lo, hi) = _takeDyn(maxBits, chunkBits);
    return joinInt64(lo, hi);
  }

  /// `uint_XdY` with `X = maxBits <= 64` as [BigInt] (spec 2.4).
  BigInt readDynUintBigInt(int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits, 64);
    final (lo, hi) = _takeDyn(maxBits, chunkBits);
    return joinBigInt(lo, hi);
  }

  /// `int_XdY` with `X = maxBits <= 32` (spec 2.5). Range and negative zero
  /// as [readInt].
  int readDynInt(int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits, 32);
    final neg = readBool();
    final (lo, hi) = _takeDyn(maxBits, chunkBits);
    return _signedInt(neg, lo, hi, maxBits, 'int_${maxBits}d$chunkBits');
  }

  /// `int_XdY` with `X = maxBits <= 64` as [Int64] (spec 2.5).
  Int64 readDynIntInt64(int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits, 64);
    final neg = readBool();
    final (lo, hi) = _takeDyn(maxBits, chunkBits);
    return _signedInt64(neg, lo, hi, maxBits, 'int_${maxBits}d$chunkBits');
  }

  // --------------------------------------------------------------- floats

  /// `float` (spec 2.6): IEEE binary32.
  double readFloat32() {
    _need(32);
    for (var i = 0; i < 4; i++) {
      _scratch.setUint8(i, _take(8));
    }
    return _scratch.getFloat32(0, Endian.little);
  }

  /// `double` (spec 2.6): IEEE binary64.
  double readFloat64() {
    _need(64);
    for (var i = 0; i < 8; i++) {
      _scratch.setUint8(i, _take(8));
    }
    return _scratch.getFloat64(0, Endian.little);
  }

  /// `fp_N[min, max]` (spec 2.7).
  FixedPoint readFixedPoint(FixedPointType type) {
    final (lo, hi) = _takeSlice(0, 0, 0, type.bits);
    return type.decodeRaw(joinInt(lo, hi));
  }

  // ------------------------------------------------------ strings / bytes

  /// Reads a `uint_32dY` count of elements with at least [minBits] bits
  /// each. A count above [maxLength] (if given) is invalidValue (spec 2.16,
  /// checked first); then `count * max(1, minBits) <= remainingBits` is
  /// checked (spec section 3), failing with unexpectedEnd before anything is
  /// allocated.
  int _readCount(int chunkBits, int minBits, int? maxLength) {
    _checkDyn(32, chunkBits, 32);
    _checkMaxLength(maxLength);
    final (lo, _) = _takeDyn(32, chunkBits);
    if (maxLength != null && lo > maxLength) {
      _fail(
        BitisDecodeErrorKind.invalidValue,
        'count $lo exceeds the maximum length $maxLength',
      );
    }
    final per = minBits < 1 ? 1 : minBits;
    if (lo > remainingBits ~/ per) {
      _fail(
        BitisDecodeErrorKind.unexpectedEnd,
        'count $lo x $per bits exceeds the $remainingBits remaining bits',
      );
    }
    return lo;
  }

  static void _checkMaxLength(int? maxLength) {
    if (maxLength != null && maxLength < 1) {
      throw ArgumentError.value(maxLength, 'maxLength', 'must be >= 1');
    }
  }

  Uint8List _takeBytes(int n) {
    final out = Uint8List(n);
    if (_bit == 0) {
      out.setRange(0, n, _data, _byte);
      _byte += n;
    } else {
      for (var i = 0; i < n; i++) {
        out[i] = _take(8);
      }
    }
    return out;
  }

  /// `astr_dY` / `astr_dY[M]` (spec 2.8, 2.16). A byte >= 0x80 or more than
  /// [maxLength] bytes (`M`, if given) is [BitisDecodeErrorKind.invalidValue].
  String readAStr(int chunkBits, {int? maxLength}) {
    final n = _readCount(chunkBits, 8, maxLength);
    final bytes = _takeBytes(n);
    for (final b in bytes) {
      if (b >= 0x80) {
        _fail(BitisDecodeErrorKind.invalidValue, 'astr byte $b is not ASCII');
      }
    }
    return String.fromCharCodes(bytes);
  }

  /// `binary_dY` / `binary_dY[M]` (spec 2.9, 2.16). More than [maxLength]
  /// bytes (`M`, if given) is [BitisDecodeErrorKind.invalidValue].
  Uint8List readBinary(int chunkBits, {int? maxLength}) =>
      _takeBytes(_readCount(chunkBits, 8, maxLength));

  // ------------------------------------------------------------ composites

  /// `optional T` (spec 2.10): returns null if the presence bit is 0.
  T? readOptional<T extends Object>(BitisReadFn<T> readValue) =>
      readBool() ? readValue(this) : null;

  /// `repeated_fixed_N T` (spec 2.11): reads exactly [length] elements.
  /// [length] comes from the schema (trusted).
  List<T> readFixedArray<T>(int length, BitisReadFn<T> readElement) {
    if (length < 0) throw ArgumentError.value(length, 'length');
    final out = <T>[];
    for (var i = 0; i < length; i++) {
      out.add(readElement(this));
    }
    return out;
  }

  /// `repeated_dyn_Y T` / `repeated_dyn_Y[M] T` (spec 2.12, 2.16).
  /// [elementMinBits] is `min_bits(T)` (see [MinBits]); a count above
  /// [maxLength] (`M`, if given) is [BitisDecodeErrorKind.invalidValue], then
  /// the count is checked against the remaining input before any element is
  /// read.
  List<T> readDynArray<T>(
    int chunkBits,
    int elementMinBits,
    BitisReadFn<T> readElement, {
    int? maxLength,
  }) {
    final n = _readCount(chunkBits, elementMinBits, maxLength);
    final out = <T>[];
    for (var i = 0; i < n; i++) {
      out.add(readElement(this));
    }
    return out;
  }

  int _readIndex(int count, int chunkBits, String what) {
    _checkDyn(32, chunkBits, 32);
    final (lo, _) = _takeDyn(32, chunkBits);
    if (lo >= count) {
      _fail(
        BitisDecodeErrorKind.invalidDiscriminant,
        '$what index $lo, only $count variants',
      );
    }
    return lo;
  }

  /// `enum E(Y)` (spec 2.13): returns the zero-based index; an index
  /// `>= count` is [BitisDecodeErrorKind.invalidDiscriminant].
  int readEnumIndex(int count, int chunkBits) =>
      _readIndex(count, chunkBits, 'enum');

  /// `oneof name(Y)` (spec 2.14): returns the zero-based variant index; an
  /// index `>= count` is [BitisDecodeErrorKind.invalidDiscriminant]. The
  /// caller reads the payload next.
  int readOneOfIndex(int count, int chunkBits) =>
      _readIndex(count, chunkBits, 'oneof');
}
