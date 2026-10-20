// Test-only interpreter for compact type specs (as used by the Rust
// fixtures): bool, u<X>, i<X>, du<X>d<Y>, di<X>d<Y>, f32, f64,
// fp<N>[min,max], astr<Y>, bin<Y>, opt<T>, fa<N><T>, da<Y><T>.
//
// Maps each spec to the BitWriter/BitReader calls generated code would use,
// with the agreed Dart type mapping (int <= 32 bits, Int64 33..64 signed and
// unsigned <= 63, BigInt for unsigned 64).

import 'dart:math';
import 'dart:typed_data';

import 'package:bitis/bitis.dart';

/// Parsed type spec.
sealed class TS {
  const TS();

  /// Parses a spec string.
  static TS parse(String s) {
    final p = _Parser(s);
    final t = p.type();
    if (p.i != s.length) throw FormatException('trailing spec', s, p.i);
    return t;
  }

  /// Writes [v] (Dart value).
  void write(BitWriter w, Object? v);

  /// Reads a Dart value.
  Object? read(BitReader r);

  /// min_bits per spec section 4.
  int get minBits;

  /// Converts a fixture value (strings / lists / null) to the Dart value
  /// that is written.
  Object? input(Object? fixture);

  /// Converts a fixture value to the Dart value expected after decoding.
  Object? expected(Object? fixture) => input(fixture);

  /// Value equality (bit-exact for floats).
  bool same(Object? a, Object? b) => a == b;

  /// A random valid value.
  Object? random(Random rnd);
}

final class _Parser {
  _Parser(this.s);
  final String s;
  int i = 0;

  bool eat(String t) {
    if (s.startsWith(t, i)) {
      i += t.length;
      return true;
    }
    return false;
  }

  int number() {
    final start = i;
    if (i < s.length && s[i] == '-') i++;
    while (i < s.length && s.codeUnitAt(i) >= 48 && s.codeUnitAt(i) <= 57) {
      i++;
    }
    return int.parse(s.substring(start, i));
  }

  TS inner() {
    if (!eat('<')) throw FormatException('expected <', s, i);
    final t = type();
    if (!eat('>')) throw FormatException('expected >', s, i);
    return t;
  }

  TS type() {
    if (eat('bool')) return const BoolT();
    if (eat('f32')) return const FloatT(32);
    if (eat('f64')) return const FloatT(64);
    if (eat('fp')) {
      final n = number();
      eat('[');
      final a = number();
      eat(',');
      final b = number();
      eat(']');
      return FpT(FixedPointType(n, a, b));
    }
    if (eat('astr')) return AStrT(number());
    if (eat('bin')) return BinT(number());
    if (eat('opt')) return OptT(inner());
    if (eat('fa')) {
      final n = number();
      return FaT(n, inner());
    }
    if (eat('da')) {
      final y = number();
      return DaT(y, inner());
    }
    if (eat('du') || eat('di')) {
      final signed = s[i - 1] == 'i';
      final x = number();
      eat('d');
      return DynT(signed, x, number());
    }
    if (eat('u') || eat('i')) {
      final signed = s[i - 1] == 'i';
      return IntT(signed, number());
    }
    throw FormatException('bad spec', s, i);
  }
}

BigInt _randBig(Random rnd, int bits) {
  var v = BigInt.zero;
  // pick a random bit length to get small and large values
  final len = rnd.nextInt(bits + 1);
  for (var k = 0; k < len; k++) {
    if (rnd.nextBool() || k == len - 1) v |= BigInt.one << k;
  }
  return v;
}

/// Kind of Dart representation of an integer field.
enum Repr { int_, int64, bigInt }

Repr reprFor(bool signed, int x) {
  if (x <= 32) return Repr.int_;
  if (signed || x <= 63) return Repr.int64;
  return Repr.bigInt;
}

Object convertInt(BigInt v, Repr repr) => switch (repr) {
      Repr.int_ => v.toInt(),
      Repr.int64 => Int64.parseInt(v.toString()),
      Repr.bigInt => v,
    };

BigInt toBig(Object? v) => switch (v) {
      final int i => BigInt.from(i),
      final Int64 i => BigInt.parse(i.toString()),
      final BigInt b => b,
      _ => throw ArgumentError(v),
    };

final class BoolT extends TS {
  const BoolT();
  @override
  void write(BitWriter w, Object? v) => w.writeBool(v! as bool);
  @override
  Object? read(BitReader r) => r.readBool();
  @override
  int get minBits => MinBits.boolean;
  @override
  Object? input(Object? f) => f == 'true';
  @override
  Object? random(Random rnd) => rnd.nextBool();
}

final class IntT extends TS {
  const IntT(this.signed, this.x);
  final bool signed;
  final int x;
  Repr get repr => reprFor(signed, x);

  BigInt get maxMag {
    final b = x <= 16 ? 16 : (x <= 32 ? 32 : 64);
    final m = x < b - 1 ? x : b - 1;
    return signed
        ? (BigInt.one << m) - BigInt.one
        : (BigInt.one << x) - BigInt.one;
  }

  @override
  void write(BitWriter w, Object? v) {
    switch ((signed, repr)) {
      case (false, Repr.int_):
        w.writeUint(v! as int, x);
      case (false, Repr.int64):
        w.writeUintInt64(v! as Int64, x);
      case (false, Repr.bigInt):
        w.writeUintBigInt(v! as BigInt, x);
      case (true, Repr.int_):
        w.writeInt(v! as int, x);
      case (true, _):
        w.writeIntInt64(v! as Int64, x);
    }
  }

  @override
  Object? read(BitReader r) => switch ((signed, repr)) {
        (false, Repr.int_) => r.readUint(x),
        (false, Repr.int64) => r.readUintInt64(x),
        (false, Repr.bigInt) => r.readUintBigInt(x),
        (true, Repr.int_) => r.readInt(x),
        (true, _) => r.readIntInt64(x),
      };

  @override
  int get minBits => signed ? MinBits.sint(x) : MinBits.uint(x);

  @override
  Object? input(Object? f) => convertInt(BigInt.parse(f! as String), repr);

  @override
  Object? random(Random rnd) {
    var v = _randBig(rnd, x);
    if (v > maxMag) v = maxMag;
    if (signed && rnd.nextBool()) v = -v;
    return convertInt(v, repr);
  }
}

final class DynT extends IntT {
  const DynT(super.signed, super.x, this.y);
  final int y;

  @override
  void write(BitWriter w, Object? v) {
    switch ((signed, repr)) {
      case (false, Repr.int_):
        w.writeDynUint(v! as int, x, y);
      case (false, Repr.int64):
        w.writeDynUintInt64(v! as Int64, x, y);
      case (false, Repr.bigInt):
        w.writeDynUintBigInt(v! as BigInt, x, y);
      case (true, Repr.int_):
        w.writeDynInt(v! as int, x, y);
      case (true, _):
        w.writeDynIntInt64(v! as Int64, x, y);
    }
  }

  @override
  Object? read(BitReader r) => switch ((signed, repr)) {
        (false, Repr.int_) => r.readDynUint(x, y),
        (false, Repr.int64) => r.readDynUintInt64(x, y),
        (false, Repr.bigInt) => r.readDynUintBigInt(x, y),
        (true, Repr.int_) => r.readDynInt(x, y),
        (true, _) => r.readDynIntInt64(x, y),
      };

  @override
  int get minBits => signed ? MinBits.dynInt : MinBits.dynUint;
}

/// Bit pattern of a double as hex (16 or 8 digits).
String floatHex(double v, int bits) {
  final bd = ByteData(8);
  if (bits == 32) {
    bd.setFloat32(0, v);
    return bd.getUint32(0).toRadixString(16).padLeft(8, '0');
  }
  bd.setFloat64(0, v);
  return bd.getUint32(0).toRadixString(16).padLeft(8, '0') +
      bd.getUint32(4).toRadixString(16).padLeft(8, '0');
}

/// Double from a hex bit pattern.
double floatFromHex(String hex, int bits) {
  final bd = ByteData(8);
  if (bits == 32) {
    bd.setUint32(0, int.parse(hex, radix: 16));
    return bd.getFloat32(0);
  }
  bd.setUint32(0, int.parse(hex.substring(0, 8), radix: 16));
  bd.setUint32(4, int.parse(hex.substring(8), radix: 16));
  return bd.getFloat64(0);
}

final class FloatT extends TS {
  const FloatT(this.bits);
  final int bits;
  @override
  void write(BitWriter w, Object? v) =>
      bits == 32 ? w.writeFloat32(v! as double) : w.writeFloat64(v! as double);
  @override
  Object? read(BitReader r) => bits == 32 ? r.readFloat32() : r.readFloat64();
  @override
  int get minBits => bits == 32 ? MinBits.float32 : MinBits.float64;
  @override
  Object? input(Object? f) => floatFromHex(f! as String, bits);
  @override
  bool same(Object? a, Object? b) {
    final x = a! as double;
    final y = b! as double;
    if (x.isNaN || y.isNaN) return x.isNaN && y.isNaN;
    return floatHex(x, bits) == floatHex(y, bits);
  }

  @override
  Object? random(Random rnd) {
    final bd = ByteData(8);
    for (var i = 0; i < 8; i++) {
      bd.setUint8(i, rnd.nextInt(256));
    }
    return bits == 32 ? bd.getFloat32(0) : bd.getFloat64(0);
  }
}

final class FpT extends TS {
  const FpT(this.t);
  final FixedPointType t;

  static FixedPoint _p(String s) => switch (s) {
        'U' => const FixedPoint.underflow(),
        'O' => const FixedPoint.overflow(),
        _ => FixedPoint.value(floatFromHex(s, 64)),
      };

  @override
  void write(BitWriter w, Object? v) => w.writeFixedPoint(v! as FixedPoint, t);
  @override
  Object? read(BitReader r) => r.readFixedPoint(t);
  @override
  int get minBits => MinBits.fixedPoint(t.bits);
  @override
  Object? input(Object? f) => _p((f! as String).split('>')[0]);
  @override
  Object? expected(Object? f) => _p((f! as String).split('>')[1]);
  @override
  bool same(Object? a, Object? b) {
    final x = a! as FixedPoint;
    final y = b! as FixedPoint;
    if (x.state != y.state) return false;
    if (!x.isValue) return true;
    return floatHex(x.value, 64) == floatHex(y.value, 64);
  }

  @override
  Object? random(Random rnd) {
    final k = rnd.nextInt(10);
    if (k == 0) return const FixedPoint.underflow();
    if (k == 1) return const FixedPoint.overflow();
    return t.fromDouble(
      t.min + rnd.nextDouble() * (t.max.toDouble() - t.min.toDouble()),
    );
  }
}

final class AStrT extends TS {
  const AStrT(this.y);
  final int y;
  @override
  void write(BitWriter w, Object? v) => w.writeAStr(v! as String, y);
  @override
  Object? read(BitReader r) => r.readAStr(y);
  @override
  int get minBits => MinBits.astr;
  @override
  Object? input(Object? f) => f;
  @override
  Object? random(Random rnd) => String.fromCharCodes(
      List.generate(rnd.nextInt(12), (_) => rnd.nextInt(128)));
}

Uint8List hexToBytes(String hex) => Uint8List.fromList([
      for (var i = 0; i < hex.length; i += 2)
        int.parse(hex.substring(i, i + 2), radix: 16),
    ]);

String bytesToHex(List<int> b) =>
    b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

final class BinT extends TS {
  const BinT(this.y);
  final int y;
  @override
  void write(BitWriter w, Object? v) => w.writeBinary(v! as Uint8List, y);
  @override
  Object? read(BitReader r) => r.readBinary(y);
  @override
  int get minBits => MinBits.binary;
  @override
  Object? input(Object? f) => hexToBytes(f! as String);
  @override
  bool same(Object? a, Object? b) =>
      bytesToHex(a! as Uint8List) == bytesToHex(b! as Uint8List);
  @override
  Object? random(Random rnd) => Uint8List.fromList(
        List.generate(rnd.nextInt(12), (_) => rnd.nextInt(256)),
      );
}

final class OptT extends TS {
  const OptT(this.t);
  final TS t;
  @override
  void write(BitWriter w, Object? v) =>
      w.writeOptional<Object>(v, (w, x) => t.write(w, x));
  @override
  Object? read(BitReader r) => r.readOptional<Object>((r) => t.read(r)!);
  @override
  int get minBits => MinBits.optional;
  @override
  Object? input(Object? f) => f == null ? null : t.input(f);
  @override
  Object? expected(Object? f) => f == null ? null : t.expected(f);
  @override
  bool same(Object? a, Object? b) =>
      (a == null || b == null) ? a == b : t.same(a, b);
  @override
  Object? random(Random rnd) => rnd.nextBool() ? null : t.random(rnd);
}

final class FaT extends TS {
  const FaT(this.n, this.t);
  final int n;
  final TS t;
  @override
  void write(BitWriter w, Object? v) =>
      w.writeFixedArray<Object?>(v! as List<Object?>, n, t.write);
  @override
  Object? read(BitReader r) => r.readFixedArray<Object?>(n, t.read);
  @override
  int get minBits => MinBits.fixedArray(n, t.minBits);
  @override
  Object? input(Object? f) => [for (final e in f! as List) t.input(e)];
  @override
  Object? expected(Object? f) => [for (final e in f! as List) t.expected(e)];
  @override
  bool same(Object? a, Object? b) {
    final x = a! as List;
    final y = b! as List;
    if (x.length != y.length) return false;
    for (var i = 0; i < x.length; i++) {
      if (!t.same(x[i], y[i])) return false;
    }
    return true;
  }

  @override
  Object? random(Random rnd) => [for (var i = 0; i < n; i++) t.random(rnd)];
}

final class DaT extends FaT {
  const DaT(this.y, TS t) : super(-1, t);
  final int y;
  @override
  void write(BitWriter w, Object? v) =>
      w.writeDynArray<Object?>(v! as List<Object?>, y, t.write);
  @override
  Object? read(BitReader r) => r.readDynArray<Object?>(y, t.minBits, t.read);
  @override
  int get minBits => MinBits.dynArray;
  @override
  Object? random(Random rnd) =>
      [for (var i = rnd.nextInt(5); i > 0; i--) t.random(rnd)];
}
