// Exact bit-level vectors for every type, derived from docs/WIRE_FORMAT.md.
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

import 'bits.dart';

BigInt big(String s) => BigInt.parse(s);
Int64 i64(String s) => Int64.parseInt(s);
BigInt p2(int k) => BigInt.one << k;

void main() {
  group('bit order', () {
    test('LSB first within bytes, values LSB first', () {
      final w = BitWriter()
        ..writeUint(1, 1)
        ..writeUint(0, 1)
        ..writeUint(6, 3) // 011
        ..writeUint(0x1FF, 9);
      expect(w.bitLength, 14);
      // stream: 1 0 011 111111111 -> byte0 = bits 0..7 = 1,0,0,1,1,1,1,1
      expect(w.toBytes(), [0xF9, 0x3F]);
    });

    test('empty writer encodes to 0 bytes', () {
      expect(BitWriter().toBytes(), isEmpty);
      expect(BitWriter().bitLength, 0);
    });
  });

  group('bool', () {
    test('vectors', () {
      expectWrites((w) => w.writeBool(true), '1');
      expectWrites((w) => w.writeBool(false), '0');
      expect(readsAll('1', (r) => r.readBool()), isTrue);
      expect(readsAll('0', (r) => r.readBool()), isFalse);
    });
  });

  group('uint_X', () {
    test('int variant boundaries (X <= 32)', () {
      for (final x in [1, 2, 7, 8, 9, 15, 16, 17, 31, 32]) {
        final max = (p2(x) - BigInt.one).toInt();
        final cases = <int>{0, 1, max, if (x > 1) (p2(x - 1)).toInt()};
        for (final v in cases) {
          expectWrites((w) => w.writeUint(v, x), lsb(v, x));
          expect(readsAll(lsb(v, x), (r) => r.readUint(x)), v);
        }
      }
      expectWrites((w) => w.writeUint(17, 5), '10001');
    });

    test('Int64 variant boundaries (X <= 63)', () {
      for (final x in [1, 31, 32, 33, 52, 53, 54, 62, 63]) {
        for (final b in [
          BigInt.zero,
          BigInt.one,
          p2(x) - BigInt.one,
          p2(x - 1),
          if (x > 32) p2(32),
          if (x > 32) p2(32) - BigInt.one,
        ]) {
          final v = i64(b.toString());
          expectWrites((w) => w.writeUintInt64(v, x), lsb(b, x));
          expect(readsAll(lsb(b, x), (r) => r.readUintInt64(x)), v);
        }
      }
    });

    test('BigInt variant (uint_64)', () {
      for (final b in [
        BigInt.zero,
        BigInt.one,
        p2(64) - BigInt.one,
        p2(63),
        p2(63) - BigInt.one,
        p2(53) + BigInt.one,
        big('0x0123456789abcdef'),
      ]) {
        expectWrites((w) => w.writeUintBigInt(b, 64), lsb(b, 64));
        expect(readsAll(lsb(b, 64), (r) => r.readUintBigInt(64)), b);
      }
    });
  });

  group('int_X (sign-magnitude)', () {
    test('vectors', () {
      expectWrites((w) => w.writeInt(-5, 5), '1 10100');
      expectWrites((w) => w.writeInt(5, 5), '0 10100');
      expectWrites((w) => w.writeInt(0, 5), '0 00000');
      expectWrites((w) => w.writeInt(-1, 1), '1 1');
    });

    test('int variant range boundaries', () {
      for (final x in [1, 2, 8, 15, 16, 17, 31, 32]) {
        final b = x <= 16 ? 16 : 32;
        final m = x < b - 1 ? x : b - 1;
        final max = (p2(m) - BigInt.one).toInt();
        for (final v in [0, 1, -1, max, -max]) {
          final s = '${v < 0 ? 1 : 0} ${lsb(v.abs(), x)}';
          expectWrites((w) => w.writeInt(v, x), s);
          expect(readsAll(s, (r) => r.readInt(x)), v);
        }
      }
    });

    test('Int64 variant range boundaries', () {
      for (final x in [5, 16, 32, 33, 53, 54, 63, 64]) {
        final b = x <= 16 ? 16 : (x <= 32 ? 32 : 64);
        final m = x < b - 1 ? x : b - 1;
        final max = p2(m) - BigInt.one;
        for (final v in [BigInt.zero, BigInt.one, -BigInt.one, max, -max]) {
          final s = '${v.isNegative ? 1 : 0} ${lsb(v.abs(), x)}';
          final iv = i64(v.toString());
          expectWrites((w) => w.writeIntInt64(iv, x), s);
          expect(readsAll(s, (r) => r.readIntInt64(x)), iv);
        }
      }
    });

    test('negative zero decodes to 0', () {
      final v = readsAll('1 00000', (r) => r.readInt(5));
      expect(v, 0);
      expect(v.isNegative, isFalse);
      expect(readsAll('1 ${'0' * 64}', (r) => r.readIntInt64(64)), Int64.ZERO);
      expect(readsAll('1 0', (r) => r.readDynInt(32, 3)), 0);
      expect(readsAll('1 0', (r) => r.readDynInt(32, 3)).isNegative, isFalse);
      expect(readsAll('1 0', (r) => r.readDynIntInt64(64, 8)), Int64.ZERO);
      // non-minimal negative zero
      expect(readsAll('1 1 000 0', (r) => r.readDynInt(32, 3)), 0);
    });
  });

  group('uint_XdY', () {
    test('spec example and small values', () {
      expectWrites((w) => w.writeDynUint(10, 32, 3), '1 010 1 100 0');
      expectWrites((w) => w.writeDynUint(0, 32, 3), '0');
      expectWrites((w) => w.writeDynUint(1, 32, 3), '1 100 0');
      expectWrites((w) => w.writeDynUint(7, 32, 3), '1 111 0');
      expectWrites((w) => w.writeDynUint(8, 32, 3), '1 000 1 100 0');
      expect(readsAll('1 010 1 100 0', (r) => r.readDynUint(32, 3)), 10);
      expect(readsAll('0', (r) => r.readDynUint(32, 3)), 0);
    });

    test('no continuation bit when X is used up', () {
      expectWrites((w) => w.writeDynUint(255, 8, 4), '1 1111 1 1111');
      expectWrites((w) => w.writeDynUint(255, 8, 3), '1 111 1 111 1 11');
      expectWrites((w) => w.writeDynUint(15, 8, 4), '1 1111 0');
      expectWrites((w) => w.writeDynUint(16, 8, 4), '1 0000 1 1000');
      expectWrites((w) => w.writeDynUint(1, 1, 1), '1 1');
      expectWrites((w) => w.writeDynUint(0, 1, 1), '0');
      expect(readsAll('1 111 1 111 1 11', (r) => r.readDynUint(8, 3)), 255);
      expect(readsAll('1 1111 1 1111', (r) => r.readDynUint(8, 4)), 255);
    });

    test('chunk = X (single chunk)', () {
      final max32 = (p2(32) - BigInt.one).toInt();
      expectWrites((w) => w.writeDynUint(max32, 32, 32), '1 ${'1' * 32}');
      final s64 = '1 ${'1' * 64}';
      expectWrites(
          (w) => w.writeDynUintBigInt(p2(64) - BigInt.one, 64, 64), s64);
      expect(readsAll(s64, (r) => r.readDynUintBigInt(64, 64)),
          p2(64) - BigInt.one);
      expectWrites(
          (w) => w.writeDynUintBigInt(BigInt.one, 64, 64), '1 1${'0' * 63}');
    });

    test('64-bit values cross the 32-bit boundary', () {
      // uint_64d33: 2^33 -> chunk0 (33 bits) = 0, cont, chunk1 (31 bits) = 1
      expectWrites(
        (w) => w.writeDynUintBigInt(p2(33), 64, 33),
        '1 ${lsb(0, 33)} 1 ${lsb(1, 31)}',
      );
      final v = p2(64) - BigInt.one;
      final s = '1 ${List.filled(8, '11111111').join(' 1 ')}';
      expectWrites((w) => w.writeDynUintBigInt(v, 64, 8), s);
      expect(readsAll(s, (r) => r.readDynUintBigInt(64, 8)), v);
      final i = i64('9223372036854775807');
      final si = '1 ${List.filled(9, '1111111').join(' 1 ')}';
      expectWrites((w) => w.writeDynUintInt64(i, 63, 7), si);
      expect(readsAll(si, (r) => r.readDynUintInt64(63, 7)), i);
    });

    test('non-minimal encodings are accepted', () {
      expect(readsAll('1 000 1 000 0', (r) => r.readDynUint(32, 3)), 0);
      expect(readsAll('1 010 1 000 0', (r) => r.readDynUint(32, 3)), 2);
      expect(readsAll('1 0000 1 0000', (r) => r.readDynUint(8, 4)), 0);
    });
  });

  group('int_XdY', () {
    test('vectors', () {
      expectWrites((w) => w.writeDynInt(0, 32, 3), '0 0');
      expectWrites((w) => w.writeDynInt(-10, 32, 3), '1 1 010 1 100 0');
      expectWrites((w) => w.writeDynInt(10, 32, 3), '0 1 010 1 100 0');
      expect(readsAll('1 1 010 1 100 0', (r) => r.readDynInt(32, 3)), -10);
      expectWrites((w) => w.writeDynInt(-32767, 16, 8),
          '1 1 ${lsb(255, 8)} 1 ${lsb(127, 8)}');
      final m = i64('-9223372036854775807');
      final s = '1 1 ${List.filled(7, '11111111').join(' 1 ')} 1 11111110';
      expectWrites((w) => w.writeDynIntInt64(m, 64, 8), s);
      expect(readsAll(s, (r) => r.readDynIntInt64(64, 8)), m);
    });
  });

  group('float / double', () {
    void f32(double v, int bits) {
      expectWrites((w) => w.writeFloat32(v), lsb(bits, 32));
    }

    void f64(double v, String hexBits) {
      expectWrites((w) => w.writeFloat64(v), lsb(big('0x$hexBits'), 64));
    }

    test('float32 vectors', () {
      f32(0.0, 0);
      f32(-0.0, 0x80000000);
      f32(1.0, 0x3F800000);
      f32(-2.25, 0xC0100000);
      f32(double.infinity, 0x7F800000);
      f32(double.negativeInfinity, 0xFF800000);
      f32(double.nan, 0x7FC00000);
      f32(-double.nan, 0x7FC00000);
      f32(1.401298464324817e-45, 1);
      f32(3.4028234663852886e38, 0x7F7FFFFF);
    });

    test('float64 vectors', () {
      f64(0.0, '0000000000000000');
      f64(-0.0, '8000000000000000');
      f64(1.0, '3ff0000000000000');
      f64(double.infinity, '7ff0000000000000');
      f64(double.negativeInfinity, 'fff0000000000000');
      f64(double.nan, '7ff8000000000000');
      f64(-double.nan, '7ff8000000000000');
      f64(5e-324, '0000000000000001');
      f64(double.maxFinite, '7fefffffffffffff');
    });

    test('decode special values', () {
      final nz = readsAll(lsb(0x80000000, 32), (r) => r.readFloat32());
      expect(nz, 0.0);
      expect(nz.isNegative, isTrue);
      expect(
          readsAll(lsb(0x7FC00001, 32), (r) => r.readFloat32()).isNaN, isTrue);
      expect(
          readsAll(lsb(0xFFC00000, 32), (r) => r.readFloat32()).isNaN, isTrue);
      expect(readsAll(lsb(0x7F800000, 32), (r) => r.readFloat32()),
          double.infinity);
      final nz64 = readsAll(lsb(p2(63), 64), (r) => r.readFloat64());
      expect(nz64.isNegative && nz64 == 0.0, isTrue);
      expect(
          readsAll(lsb(big('0xfff8000000000001'), 64), (r) => r.readFloat64())
              .isNaN,
          isTrue);
      expect(readsAll(lsb(1, 64), (r) => r.readFloat64()), 5e-324);
    });

    test('unaligned float', () {
      expectWrites(
        (w) => w
          ..writeBool(true)
          ..writeFloat64(-1.5)
          ..writeUint(2, 3),
        '1 ${lsb(big('0xbff8000000000000'), 64)} 010',
      );
    });
  });

  group('fixed point', () {
    const t8 = FixedPointType(8, 0, 253); // S = R = 253: t = v + 1
    test('rounding of exact .5 ties (half away from zero)', () {
      for (final (v, raw) in [
        (0.5, 2),
        (1.5, 3),
        (2.5, 4),
        (251.5, 253),
        (252.5, 254)
      ]) {
        expectWrites(
            (w) => w.writeFixedPoint(FixedPoint.value(v), t8), lsb(raw, 8));
      }
    });

    test('under/overflow states and out-of-range values', () {
      expectWrites((w) => w.writeFixedPoint(const FixedPoint.underflow(), t8),
          '00000000');
      expectWrites((w) => w.writeFixedPoint(const FixedPoint.overflow(), t8),
          '11111111');
      expectWrites(
          (w) => w.writeFixedPoint(const FixedPoint.value(253.0001), t8),
          '11111111');
      expectWrites(
          (w) => w.writeFixedPoint(const FixedPoint.value(-0.0001), t8),
          '00000000');
      expectWrites(
          (w) => w.writeFixedPoint(const FixedPoint.value(double.nan), t8),
          '00000000');
      expectWrites(
          (w) => w.writeFixedPoint(const FixedPoint.value(double.infinity), t8),
          '11111111');
      expectWrites(
        (w) => w.writeFixedPoint(
            const FixedPoint.value(double.negativeInfinity), t8),
        '00000000',
      );
      expectWrites(
          (w) => w.writeFixedPoint(const FixedPoint.value(0), t8), lsb(1, 8));
      expectWrites((w) => w.writeFixedPoint(const FixedPoint.value(-0.0), t8),
          lsb(1, 8));
      expectWrites((w) => w.writeFixedPoint(const FixedPoint.value(253), t8),
          lsb(254, 8));
      expect(readsAll('00000000', (r) => r.readFixedPoint(t8)),
          const FixedPoint.underflow());
      expect(readsAll('11111111', (r) => r.readFixedPoint(t8)),
          const FixedPoint.overflow());
      expect(readsAll(lsb(1, 8), (r) => r.readFixedPoint(t8)),
          const FixedPoint.value(0));
    });

    test('decode formula', () {
      const t = FixedPointType(10, -5, 100);
      for (var raw = 1; raw < 1023; raw++) {
        final v = readsAll(lsb(raw, 10), (r) => r.readFixedPoint(t));
        expect(v.value, (raw - 1).toDouble() / 1021.0 * 105.0 + -5.0);
        // encode(decode(raw)) == raw
        expect(t.encodeRaw(v), raw);
      }
    });

    test('fp_2 has exactly two values', () {
      const t = FixedPointType(2, 0, 1);
      expectWrites(
          (w) => w.writeFixedPoint(const FixedPoint.value(0), t), '10');
      expectWrites(
          (w) => w.writeFixedPoint(const FixedPoint.value(1), t), '01');
      expectWrites(
          (w) => w.writeFixedPoint(const FixedPoint.value(0.5), t), '01');
    });

    test('fp_53 raw values beyond 32 bits', () {
      const t = FixedPointType(53, -1000000, 1000000);
      expectWrites(
          (w) => w.writeFixedPoint(const FixedPoint.overflow(), t), '1' * 53);
      final raw = t.encodeRaw(const FixedPoint.value(1000000));
      expect(raw, 9007199254740990); // 2^53 - 2
      expectWrites((w) => w.writeFixedPoint(const FixedPoint.value(1000000), t),
          lsb(raw, 53));
    });
  });

  group('astr / binary', () {
    test('astr', () {
      // count 2 as uint_32d4: 1 0100 0, then 'H' 'i'
      final s = '1 0100 0 ${lsb(0x48, 8)} ${lsb(0x69, 8)}';
      expectWrites((w) => w.writeAStr('Hi', 4), s);
      expect(readsAll(s, (r) => r.readAStr(4)), 'Hi');
      expectWrites((w) => w.writeAStr('', 4), '0');
      expect(readsAll('0', (r) => r.readAStr(4)), '');
      final all = String.fromCharCodes(List.generate(128, (i) => i));
      final w = BitWriter()..writeAStr(all, 7);
      expect(BitReader(w.toBytes()).readAStr(7), all);
    });

    test('binary (aligned and unaligned)', () {
      final s =
          '1 001 0 ${lsb(0xAB, 8)} ${lsb(0x01, 8)} ${lsb(0xFF, 8)} ${lsb(0, 8)}';
      final data = Uint8List.fromList([0xAB, 0x01, 0xFF, 0]);
      expectWrites((w) => w.writeBinary(data, 3), s);
      expect(readsAll(s, (r) => r.readBinary(3)), data);
      expectWrites((w) => w.writeBinary(<int>[0xAB, 1, 255, 0], 3), s);
      // aligned body after a byte-aligned count
      final s8 = '1 ${lsb(3, 7)} 0 ${lsb(1, 8)} ${lsb(2, 8)} ${lsb(3, 8)}';
      expectWrites((w) => w.writeBinary(Uint8List.fromList([1, 2, 3]), 7), s8);
      expect(readsAll(s8, (r) => r.readBinary(7)), [1, 2, 3]);
    });
  });

  group('composites', () {
    test('optional', () {
      expectWrites(
          (w) => w.writeOptional<int>(null, (w, v) => w.writeUint(v, 5)), '0');
      expectWrites((w) => w.writeOptional<int>(17, (w, v) => w.writeUint(v, 5)),
          '1 10001');
      expect(
          readsAll('0', (r) => r.readOptional((r) => r.readUint(5))), isNull);
      expect(
          readsAll('1 10001', (r) => r.readOptional((r) => r.readUint(5))), 17);
    });

    test('fixed array', () {
      expectWrites(
          (w) => w.writeFixedArray([1, 2], 2, (w, v) => w.writeUint(v, 2)),
          '10 01');
      expect(
          readsAll('10 01', (r) => r.readFixedArray(2, (r) => r.readUint(2))),
          [1, 2]);
      expectWrites(
          (w) => w.writeFixedArray(<int>[], 0, (w, v) => w.writeUint(v, 2)),
          '');
    });

    test('dyn array', () {
      expectWrites(
          (w) => w.writeDynArray([true, false], 3, (w, v) => w.writeBool(v)),
          '1 010 0 1 0');
      expect(
        readsAll('1 010 0 1 0',
            (r) => r.readDynArray(3, MinBits.boolean, (r) => r.readBool())),
        [true, false],
      );
      expectWrites(
          (w) => w.writeDynArray(<bool>[], 3, (w, v) => w.writeBool(v)), '0');
    });

    test('enum / oneof index', () {
      expectWrites((w) => w.writeEnumIndex(2, 3, 2), '1 01 0');
      expectWrites((w) => w.writeEnumIndex(0, 3, 2), '0');
      expectWrites((w) => w.writeOneOfIndex(1, 2, 4), '1 1000 0');
      expect(readsAll('1 01 0', (r) => r.readEnumIndex(3, 2)), 2);
      expect(readsAll('1 1000 0', (r) => r.readOneOfIndex(2, 4)), 1);
    });
  });

  test('unaligned sequence of mixed types', () {
    final bits = [
      '1', // bool true
      '101', // uint_3 = 5
      '1 0011', // int_4 = -12
      '1 010 1 100 0', // uint_32d3 = 10
      lsb(0x3FC00000, 32), // float 1.5
      '1 10 0 ${lsb(0x41, 8)}', // astr_d2 "A"
      '1 1 11 0', // int_8d2 = -3
      '1 ${lsb(0xFFFFFFFF, 33)}', // optional uint_33 = 2^32 - 1
    ].join(' ');
    expectWrites(
      (w) => w
        ..writeBool(true)
        ..writeUint(5, 3)
        ..writeInt(-12, 4)
        ..writeDynUint(10, 32, 3)
        ..writeFloat32(1.5)
        ..writeAStr('A', 2)
        ..writeDynInt(-3, 8, 2)
        ..writeOptional(Int64(0xFFFFFFFF), (w, v) => w.writeUintInt64(v, 33)),
      bits,
    );
    final r = BitReader(bitsToBytes(bits));
    expect(r.readBool(), isTrue);
    expect(r.readUint(3), 5);
    expect(r.readInt(4), -12);
    expect(r.readDynUint(32, 3), 10);
    expect(r.readFloat32(), 1.5);
    expect(r.readAStr(2), 'A');
    expect(r.readDynInt(8, 2), -3);
    expect(r.readOptional((r) => r.readUintInt64(33)), Int64(0xFFFFFFFF));
    expect(r.bitPosition, clean(bits).length);
    expect(r.remainingBits, 8 * r.data.length - clean(bits).length);
  });
}
