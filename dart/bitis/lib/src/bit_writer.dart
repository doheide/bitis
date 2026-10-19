import 'dart:typed_data';

import 'package:fixnum/fixnum.dart';

import 'codec.dart';
import 'errors.dart';
import 'fixed_point.dart';
import 'int_util.dart';

/// Callback writing one element / optional payload. Generated code passes
/// closures like `(w, v) => w.writeUint(v, 5)` or `(w, v) => v.bitisWrite(w)`.
typedef BitisWriteFn<T> = void Function(BitWriter w, T value);

/// Writes a bitis bit stream (LSB first, no alignment, see
/// `docs/WIRE_FORMAT.md`).
///
/// Every `write*` method validates its value and throws
/// [BitisEncodeException] with [BitisEncodeErrorKind.valueOutOfRange] if it
/// cannot be encoded; the writer is then left in an unspecified state and
/// should be discarded. Invalid *type parameters* (bit widths outside the
/// documented range) are programming errors and throw [ArgumentError].
///
/// Naming: the plain variants (`writeUint`, `writeInt`, `writeDynUint`, ...)
/// take a Dart `int` and support widths up to 32 bits; `...Int64` variants
/// take an [Int64]; `...BigInt` variants take a [BigInt] and are used for
/// unsigned 64-bit fields.
final class BitWriter {
  /// Creates an empty writer.
  BitWriter();

  Uint8List _buf = Uint8List(32);
  int _bytes = 0; // number of complete bytes in _buf
  int _cur = 0; // partially filled byte (bits 0.._curBits-1 valid)
  int _curBits = 0; // 0..7
  final ByteData _scratch = ByteData(8);

  /// Number of bits written so far.
  int get bitLength => _bytes * 8 + _curBits;

  /// Number of bytes [toBytes] returns: `ceil(bitLength / 8)`.
  int get byteLength => _bytes + (_curBits > 0 ? 1 : 0);

  /// The encoded bytes, padded with zero bits to a byte boundary. Does not
  /// modify the writer; more data can be appended afterwards.
  Uint8List toBytes() {
    final out = Uint8List(byteLength);
    out.setRange(0, _bytes, _buf);
    if (_curBits > 0) out[_bytes] = _cur;
    return out;
  }

  void _pushByte(int b) {
    if (_bytes == _buf.length) {
      final n = Uint8List(_buf.length * 2);
      n.setRange(0, _bytes, _buf);
      _buf = n;
    }
    _buf[_bytes++] = b;
  }

  /// Writes the low [bits] bits of [value]; requires `0 <= bits <= 32` and
  /// `0 <= value < 2^bits` (unchecked).
  void _put(int value, int bits) {
    while (bits > 0) {
      final free = 8 - _curBits;
      final take = bits < free ? bits : free;
      _cur |= (value & lowMask[take]) << _curBits; // < 256
      _curBits += take;
      bits -= take;
      value = value >>> take; // value < 2^32: safe on the web
      if (_curBits == 8) {
        _pushByte(_cur);
        _cur = 0;
        _curBits = 0;
      }
    }
  }

  /// Writes [count] bits of the 64-bit value `(lo, hi)` starting at bit
  /// [shift] (LSB first). Requires `shift + count <= 64`.
  void _putSlice(int lo, int hi, int shift, int count) {
    while (count > 0) {
      int n;
      int piece;
      if (shift < 32) {
        n = 32 - shift;
        if (n > count) n = count;
        piece = (lo >>> shift) & lowMask[n];
      } else {
        n = 64 - shift;
        if (n > count) n = count;
        piece = (hi >>> (shift - 32)) & lowMask[n];
      }
      _put(piece, n);
      shift += n;
      count -= n;
    }
  }

  /// Dynamic unsigned integer body (spec 2.4) for magnitude `(lo, hi)` which
  /// must be `< 2^maxBits`.
  void _putDyn(int lo, int hi, int maxBits, int chunkBits) {
    final len = bitLengthU64(lo, hi);
    if (len == 0) {
      _put(0, 1);
      return;
    }
    _put(1, 1);
    var shift = 0;
    var remaining = maxBits;
    while (true) {
      final c = chunkBits < remaining ? chunkBits : remaining;
      _putSlice(lo, hi, shift, c);
      shift += c;
      remaining -= c;
      if (remaining == 0) break;
      final more = len > shift;
      _put(more ? 1 : 0, 1);
      if (!more) break;
    }
  }

  static Never _outOfRange(String what) =>
      throw BitisEncodeException(BitisEncodeErrorKind.valueOutOfRange, what);

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

  /// Magnitude halves of an unsigned int value with at most [bits] bits.
  static (int, int) _uintParts(int v, int bits, String type) {
    if (v < 0 || v.bitLength > bits) _outOfRange('$v does not fit $type');
    return splitInt(v);
  }

  static (int, int) _uint64Parts(Int64 v, int bits, String type) {
    if (v.isNegative || v.bitLength > bits) {
      _outOfRange('$v does not fit $type');
    }
    return splitInt64(v);
  }

  static (int, int) _bigParts(BigInt v, int bits, String type) {
    if (v.isNegative || v.bitLength > bits) {
      _outOfRange('$v does not fit $type');
    }
    return splitBigInt(v);
  }

  /// Sign and magnitude halves of a signed value (spec 2.3 range rule).
  static (bool, int, int) _intParts(int v, int bits, String type) {
    final neg = v < 0;
    final mag = neg ? -v : v;
    if (mag.bitLength > signedMagnitudeBits(bits)) {
      _outOfRange('$v does not fit $type');
    }
    final (lo, hi) = splitInt(mag);
    return (neg, lo, hi);
  }

  static (bool, int, int) _int64Parts(Int64 v, int bits, String type) {
    final neg = v.isNegative;
    if (v.isMinValue) _outOfRange('$v does not fit $type');
    final mag = neg ? -v : v;
    if (mag.bitLength > signedMagnitudeBits(bits)) {
      _outOfRange('$v does not fit $type');
    }
    final (lo, hi) = splitInt64(mag);
    return (neg, lo, hi);
  }

  // ---------------------------------------------------------------- basics

  /// Writes the low [bits] bits of [value] (`1 <= bits <= 32`,
  /// `0 <= value < 2^bits`). Same encoding as [writeUint].
  void writeBits(int value, int bits) => writeUint(value, bits);

  /// `bool` (spec 2.1): 1 bit.
  void writeBool(bool value) => _put(value ? 1 : 0, 1);

  // ------------------------------------------------------- fixed integers

  /// `uint_X` with `X = bits <= 32` (spec 2.2).
  void writeUint(int value, int bits) {
    _checkBits(bits, 32);
    final (lo, _) = _uintParts(value, bits, 'uint_$bits');
    _put(lo, bits);
  }

  /// `uint_X` with `X = bits <= 63` as [Int64] (spec 2.2).
  void writeUintInt64(Int64 value, int bits) {
    _checkBits(bits, 63);
    final (lo, hi) = _uint64Parts(value, bits, 'uint_$bits');
    _putSlice(lo, hi, 0, bits);
  }

  /// `uint_X` with `X = bits <= 64` as [BigInt] (spec 2.2); used for
  /// `uint_64`.
  void writeUintBigInt(BigInt value, int bits) {
    _checkBits(bits, 64);
    final (lo, hi) = _bigParts(value, bits, 'uint_$bits');
    _putSlice(lo, hi, 0, bits);
  }

  /// `int_X` with `X = bits <= 32`, sign-magnitude, `X + 1` bits (spec 2.3).
  /// Valid range `|value| <= min(2^X - 1, 2^(B-1) - 1)`.
  void writeInt(int value, int bits) {
    _checkBits(bits, 32);
    final (neg, lo, hi) = _intParts(value, bits, 'int_$bits');
    _put(neg ? 1 : 0, 1);
    _putSlice(lo, hi, 0, bits);
  }

  /// `int_X` with `X = bits <= 64` as [Int64] (spec 2.3).
  void writeIntInt64(Int64 value, int bits) {
    _checkBits(bits, 64);
    final (neg, lo, hi) = _int64Parts(value, bits, 'int_$bits');
    _put(neg ? 1 : 0, 1);
    _putSlice(lo, hi, 0, bits);
  }

  // ----------------------------------------------------- dynamic integers

  /// `uint_XdY` with `X = maxBits <= 32`, `Y = chunkBits` (spec 2.4).
  void writeDynUint(int value, int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits, 32);
    final (lo, hi) = _uintParts(value, maxBits, 'uint_${maxBits}d$chunkBits');
    _putDyn(lo, hi, maxBits, chunkBits);
  }

  /// `uint_XdY` with `X = maxBits <= 63` as [Int64] (spec 2.4).
  void writeDynUintInt64(Int64 value, int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits, 63);
    final (lo, hi) = _uint64Parts(value, maxBits, 'uint_${maxBits}d$chunkBits');
    _putDyn(lo, hi, maxBits, chunkBits);
  }

  /// `uint_XdY` with `X = maxBits <= 64` as [BigInt] (spec 2.4); used for
  /// `uint_64dY`.
  void writeDynUintBigInt(BigInt value, int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits, 64);
    final (lo, hi) = _bigParts(value, maxBits, 'uint_${maxBits}d$chunkBits');
    _putDyn(lo, hi, maxBits, chunkBits);
  }

  /// `int_XdY` with `X = maxBits <= 32` (spec 2.5): sign bit, then
  /// `uint_XdY` of the magnitude. Range as [writeInt].
  void writeDynInt(int value, int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits, 32);
    final (neg, lo, hi) =
        _intParts(value, maxBits, 'int_${maxBits}d$chunkBits');
    _put(neg ? 1 : 0, 1);
    _putDyn(lo, hi, maxBits, chunkBits);
  }

  /// `int_XdY` with `X = maxBits <= 64` as [Int64] (spec 2.5).
  void writeDynIntInt64(Int64 value, int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits, 64);
    final (neg, lo, hi) =
        _int64Parts(value, maxBits, 'int_${maxBits}d$chunkBits');
    _put(neg ? 1 : 0, 1);
    _putDyn(lo, hi, maxBits, chunkBits);
  }

  // --------------------------------------------------------------- floats

  /// `float` (spec 2.6): IEEE binary32, 32 bits. [value] is rounded to the
  /// nearest float. Every NaN is written as the canonical quiet NaN
  /// `0x7FC00000` (Dart cannot preserve NaN payloads on all platforms).
  void writeFloat32(double value) {
    if (value.isNaN) {
      _put(0x7FC00000, 32);
      return;
    }
    _scratch.setFloat32(0, value, Endian.little);
    for (var i = 0; i < 4; i++) {
      _put(_scratch.getUint8(i), 8);
    }
  }

  /// `double` (spec 2.6): IEEE binary64, 64 bits. Every NaN is written as
  /// the canonical quiet NaN `0x7FF8000000000000`.
  void writeFloat64(double value) {
    if (value.isNaN) {
      _put(0, 32);
      _put(0x7FF80000, 32);
      return;
    }
    _scratch.setFloat64(0, value, Endian.little);
    for (var i = 0; i < 8; i++) {
      _put(_scratch.getUint8(i), 8);
    }
  }

  /// `fp_N[min, max]` (spec 2.7): N bits.
  void writeFixedPoint(FixedPoint value, FixedPointType type) {
    final (lo, hi) = splitInt(type.encodeRaw(value));
    _putSlice(lo, hi, 0, type.bits);
  }

  // ------------------------------------------------------ strings / bytes

  void _putCount(int n, int chunkBits) {
    if (n > 0xFFFFFFFF) _outOfRange('count $n exceeds 2^32 - 1');
    _putDyn(n, 0, 32, chunkBits);
  }

  /// `astr_dY` (spec 2.8): `uint_32dY` byte count, then the ASCII bytes.
  /// Throws [BitisEncodeException] if [value] contains a character >= 0x80.
  void writeAStr(String value, int chunkBits) {
    _checkDyn(32, chunkBits, 32);
    final units = value.codeUnits;
    for (final c in units) {
      if (c >= 0x80) _outOfRange('astr contains non-ASCII character');
    }
    _putCount(units.length, chunkBits);
    for (final c in units) {
      _put(c, 8);
    }
  }

  /// `binary_dY` (spec 2.9): `uint_32dY` byte count, then the bytes.
  /// Elements of [value] must be in `0..255`.
  void writeBinary(List<int> value, int chunkBits) {
    _checkDyn(32, chunkBits, 32);
    if (value is! Uint8List) {
      for (final b in value) {
        if (b < 0 || b > 255) _outOfRange('binary element $b is not a byte');
      }
    }
    _putCount(value.length, chunkBits);
    if (_curBits == 0) {
      for (final b in value) {
        _pushByte(b);
      }
    } else {
      for (final b in value) {
        _put(b, 8);
      }
    }
  }

  // ------------------------------------------------------------ composites

  /// `optional T` (spec 2.10): presence bit, then the value if non-null.
  void writeOptional<T extends Object>(T? value, BitisWriteFn<T> writeValue) {
    if (value == null) {
      _put(0, 1);
    } else {
      _put(1, 1);
      writeValue(this, value);
    }
  }

  /// `repeated_fixed_N T` (spec 2.11): exactly [length] elements, no prefix.
  /// Throws [BitisEncodeException] if `values.length != length`.
  void writeFixedArray<T>(
    List<T> values,
    int length,
    BitisWriteFn<T> writeElement,
  ) {
    if (values.length != length) {
      _outOfRange('fixed array needs $length elements, got ${values.length}');
    }
    for (final v in values) {
      writeElement(this, v);
    }
  }

  /// `repeated_dyn_Y T` (spec 2.12): `uint_32dY` element count, then the
  /// elements.
  void writeDynArray<T>(
    List<T> values,
    int chunkBits,
    BitisWriteFn<T> writeElement,
  ) {
    _checkDyn(32, chunkBits, 32);
    _putCount(values.length, chunkBits);
    for (final v in values) {
      writeElement(this, v);
    }
  }

  void _writeIndex(int index, int count, int chunkBits, String what) {
    _checkDyn(32, chunkBits, 32);
    if (index < 0 || index >= count) {
      _outOfRange('$what index $index not in 0..${count - 1}');
    }
    _putDyn(index, 0, 32, chunkBits);
  }

  /// `enum E(Y)` (spec 2.13): `uint_32dY` of the zero-based [index]
  /// (`0 <= index < count`).
  void writeEnumIndex(int index, int count, int chunkBits) =>
      _writeIndex(index, count, chunkBits, 'enum');

  /// `oneof name(Y)` (spec 2.14): `uint_32dY` of the zero-based variant
  /// [index] (`0 <= index < count`). The caller writes the payload next.
  void writeOneOfIndex(int index, int count, int chunkBits) =>
      _writeIndex(index, count, chunkBits, 'oneof');

  /// `msg` (spec 2.15): writes [message] by calling its
  /// [BitisMessage.bitisWrite].
  void writeMessage(BitisMessage message) => message.bitisWrite(this);
}
