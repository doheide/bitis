// FixedPoint semantics, JSON helpers and MinBits.
import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:bitis/src/fixed_point.dart' show roundHalfAwayFromZero;
import 'package:test/test.dart';

double _nextUp(double x) {
  final b = ByteData(8)..setFloat64(0, x);
  var lo = b.getUint32(4);
  var hi = b.getUint32(0);
  if (lo == 0xFFFFFFFF) {
    lo = 0;
    hi++;
  } else {
    lo++;
  }
  b
    ..setUint32(0, hi)
    ..setUint32(4, lo);
  return b.getFloat64(0);
}

void main() {
  group('rounding', () {
    test('Dart double.round() is half away from zero', () {
      expect(0.5.round(), 1);
      expect(1.5.round(), 2);
      expect(2.5.round(), 3);
      expect((-2.5).round(), -3);
      expect(4503599627370495.5.round(), 4503599627370496);
    });

    test('roundHalfAwayFromZero == double.round() == Rust f64::round', () {
      final cases = <double>[
        1,
        1.5,
        2.5,
        3.5,
        1.4999999999999998,
        1.5000000000000002
      ];
      for (var k = 0; k < 53; k++) {
        final p = pow(2.0, k).toDouble();
        cases
          ..add(p)
          ..add(p + 0.5)
          ..add(_nextUp(p + 0.5))
          ..add(p - 0.5 < 1 ? 1 : p - 0.5);
      }
      final rnd = Random(3);
      for (var i = 0; i < 10000; i++) {
        cases.add(1 + rnd.nextDouble() * pow(2.0, rnd.nextInt(52)));
        cases.add(rnd.nextInt(1 << 30) + 0.5);
      }
      for (final t in cases) {
        expect(roundHalfAwayFromZero(t), t.round(), reason: '$t');
      }
      expect(roundHalfAwayFromZero(1.4999999999999998), 1);
      expect(roundHalfAwayFromZero(2.5), 3);
    });
  });

  group('FixedPoint', () {
    const t = FixedPointType(10, -5, 100);

    test('states and accessors', () {
      const u = FixedPoint.underflow();
      const o = FixedPoint.overflow();
      const v = FixedPoint.value(1.25);
      expect(u.isUnderflow && !u.isValue && !u.isOverflow, isTrue);
      expect(o.isOverflow, isTrue);
      expect(v.value, 1.25);
      expect(u.valueOrNull, isNull);
      expect(() => o.value, throwsStateError);
      expect(v, const FixedPoint.value(1.25));
      expect(v == const FixedPoint.value(1.5), isFalse);
      expect(u == o, isFalse);
      expect(v.toString(), 'FixedPoint(1.25)');
      expect(u.toString(), 'FixedPoint.underflow');
    });

    test('fromDouble classifies and quantizes', () {
      expect(t.fromDouble(100.5), const FixedPoint.overflow());
      expect(t.fromDouble(-5.5), const FixedPoint.underflow());
      expect(t.fromDouble(double.nan), const FixedPoint.underflow());
      expect(t.fromDouble(-5), const FixedPoint.value(-5));
      expect(t.fromDouble(100), const FixedPoint.value(100));
      final q = t.fromDouble(12.345);
      expect((q.value - 12.345).abs(), lessThanOrEqualTo(t.resolution / 2));
      expect(t.sameEncoding(q, const FixedPoint.value(12.345)), isTrue);
      expect(t.quantize(q), q);
      expect(t.maxRaw, 1023);
    });

    test('decode(encode(decode(r))) for large N differs by at most 1 raw', () {
      // fp_53 with a wide range: the f64 formula is not exactly invertible.
      const big = FixedPointType(53, -1000000, 1000000);
      final rnd = Random(11);
      for (var i = 0; i < 2000; i++) {
        final raw = 1 + (rnd.nextDouble() * (big.maxRaw - 2)).floor();
        final back = big.encodeRaw(big.decodeRaw(raw));
        expect((back - raw).abs(), lessThanOrEqualTo(1));
      }
      // ...while it is exact for typical sizes
      const small = FixedPointType(40, -1000000, 1000000);
      for (var i = 0; i < 2000; i++) {
        final raw = 1 + (rnd.nextDouble() * (small.maxRaw - 2)).floor();
        expect(small.encodeRaw(small.decodeRaw(raw)), raw);
      }
    });
  });

  group('BitisJson', () {
    test('Int64 / BigInt beyond 2^53 as decimal strings', () {
      expect(BitisJson.int64ToJson(Int64(5)), 5);
      expect(
          BitisJson.int64ToJson(Int64(-9007199254740991)), -9007199254740991);
      expect(BitisJson.int64ToJson(Int64.parseInt('9007199254740992')),
          '9007199254740992');
      expect(BitisJson.int64ToJson(Int64.MIN_VALUE), '-9223372036854775808');
      expect(BitisJson.int64FromJson('-9223372036854775808'), Int64.MIN_VALUE);
      expect(BitisJson.int64FromJson(42), Int64(42));
      expect(BitisJson.int64FromJson('42'), Int64(42));
      expect(() => BitisJson.int64FromJson('9223372036854775808'),
          throwsFormatException);
      expect(() => BitisJson.int64FromJson('1.5'), throwsFormatException);
      expect(() => BitisJson.int64FromJson(1.5), throwsFormatException);
      expect(() => BitisJson.int64FromJson(null), throwsFormatException);

      final max = BigInt.parse('18446744073709551615');
      expect(BitisJson.bigIntToJson(max), '18446744073709551615');
      expect(BitisJson.bigIntToJson(BigInt.from(7)), 7);
      expect(BitisJson.bigIntFromJson('18446744073709551615'), max);
      expect(BitisJson.bigIntFromJson(7), BigInt.from(7));
      // a JSON number above 2^53 is rejected when it may be inexact
      final parsed = jsonDecode('18446744073709551615');
      expect(() => BitisJson.bigIntFromJson(parsed), throwsFormatException);
    });

    test('int', () {
      expect(BitisJson.intToJson(-3), -3);
      expect(BitisJson.intFromJson(-3), -3);
      expect(BitisJson.intFromJson(jsonDecode('4.0')), 4);
      expect(BitisJson.intFromJson('12'), 12);
      expect(() => BitisJson.intFromJson('x'), throwsFormatException);
      expect(() => BitisJson.intFromJson(true), throwsFormatException);
    });

    test('roundtrip through jsonEncode / jsonDecode', () {
      final obj = {
        'i64': BitisJson.int64ToJson(Int64.MAX_VALUE),
        'u64': BitisJson.bigIntToJson(BigInt.parse('18446744073709551615')),
        'fp': BitisJson.fixedPointToJson(const FixedPoint.value(1.5)),
        'fpU': BitisJson.fixedPointToJson(const FixedPoint.underflow()),
        'fpO': BitisJson.fixedPointToJson(const FixedPoint.overflow()),
        'bin': BitisJson.binaryToJson(Uint8List.fromList([0, 1, 254, 255])),
        'nan': BitisJson.doubleToJson(double.nan),
        'inf': BitisJson.doubleToJson(double.negativeInfinity),
        'one': BitisJson.oneOfToJson('temp', 3),
      };
      final back = jsonDecode(jsonEncode(obj)) as Map<String, Object?>;
      expect(BitisJson.int64FromJson(back['i64']), Int64.MAX_VALUE);
      expect(BitisJson.bigIntFromJson(back['u64']),
          BigInt.parse('18446744073709551615'));
      expect(BitisJson.fixedPointFromJson(back['fp']),
          const FixedPoint.value(1.5));
      expect(BitisJson.fixedPointFromJson(back['fpU']),
          const FixedPoint.underflow());
      expect(BitisJson.fixedPointFromJson(back['fpO']),
          const FixedPoint.overflow());
      expect(back['fpO'], {'state': 'overflow'});
      expect(BitisJson.binaryFromJson(back['bin']), [0, 1, 254, 255]);
      expect(back['bin'], 'AAH+/w==');
      expect(BitisJson.doubleFromJson(back['nan']).isNaN, isTrue);
      expect(BitisJson.doubleFromJson(back['inf']), double.negativeInfinity);
      expect(BitisJson.oneOfFromJson(back['one']), ('temp', 3));
    });

    test('fixed point', () {
      const t = FixedPointType(8, 0, 253);
      expect(BitisJson.fixedPointFromJson(300, t), const FixedPoint.overflow());
      expect(BitisJson.fixedPointFromJson(3, t), t.fromDouble(3));
      expect(
          BitisJson.fixedPointToJson(const FixedPoint.value(double.infinity)),
          {'state': 'overflow'});
      expect(BitisJson.fixedPointToJson(const FixedPoint.value(double.nan)),
          {'state': 'underflow'});
      expect(() => BitisJson.fixedPointFromJson({'state': 'x'}),
          throwsFormatException);
      expect(() => BitisJson.fixedPointFromJson('1'), throwsFormatException);
    });

    test('astr, bool, enum, oneof, containers', () {
      expect(BitisJson.astrFromJson('abc'), 'abc');
      expect(() => BitisJson.astrFromJson('äbc'), throwsFormatException);
      expect(BitisJson.isAscii('\x7f'), isTrue);
      expect(BitisJson.boolFromJson(true), isTrue);
      expect(() => BitisJson.boolFromJson(1), throwsFormatException);
      expect(BitisJson.enumFromJson('b', {'a': 1, 'b': 2}), 2);
      expect(
          () => BitisJson.enumFromJson('c', {'a': 1}), throwsFormatException);
      expect(() => BitisJson.oneOfFromJson({'a': 1, 'b': 2}),
          throwsFormatException);
      expect(() => BitisJson.oneOfFromJson(<String, Object?>{}),
          throwsFormatException);
      expect(BitisJson.objectFromJson(jsonDecode('{"a":1}')), {'a': 1});
      expect(BitisJson.listFromJson([1, 2], 2), [1, 2]);
      expect(() => BitisJson.listFromJson([1, 2], 3), throwsFormatException);
      expect(() => BitisJson.binaryFromJson('***'), throwsFormatException);
    });
  });

  test('MinBits (spec section 4)', () {
    expect(MinBits.boolean, 1);
    expect(MinBits.uint(7), 7);
    expect(MinBits.sint(7), 8);
    expect(MinBits.dynUint, 1);
    expect(MinBits.dynInt, 2);
    expect(MinBits.float32, 32);
    expect(MinBits.float64, 64);
    expect(MinBits.fixedPoint(10), 10);
    expect(MinBits.astr, 1);
    expect(MinBits.binary, 1);
    expect(MinBits.dynArray, 1);
    expect(MinBits.optional, 1);
    expect(MinBits.enumeration, 1);
    expect(MinBits.fixedArray(3, MinBits.sint(4)), 15);
    expect(MinBits.oneOf([32, MinBits.boolean, 64]), 2);
    expect(MinBits.message([1, 5, 32]), 38);
    expect(MinBits.message([]), 0);
    expect(() => MinBits.oneOf([]), throwsArgumentError);
  });
}
