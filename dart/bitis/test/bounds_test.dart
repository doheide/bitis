// Maximum lengths `[M]` (spec 2.16) and MaxBits (spec section 4).
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

import 'bits.dart';

final _end = throwsDecode(BitisDecodeErrorKind.unexpectedEnd);
final _invalid = throwsDecode(BitisDecodeErrorKind.invalidValue);

/// Encoding of a uint_32dY count followed by [n] bytes 'a'.
Uint8List _counted(int n, int chunkBits, {int bytes = -1}) {
  final w = BitWriter()..writeDynUint(n, 32, chunkBits);
  for (var i = 0; i < (bytes < 0 ? n : bytes); i++) {
    w.writeUint(0x61, 8);
  }
  return w.toBytes();
}

void main() {
  group('decode', () {
    test('at the bound', () {
      expect(BitReader(_counted(3, 4)).readAStr(4, maxLength: 3), 'aaa');
      expect(BitReader(_counted(3, 4)).readBinary(4, maxLength: 3), [
        0x61,
        0x61,
        0x61,
      ]);
      expect(
        BitReader(_counted(3, 4))
            .readDynArray(4, 8, (r) => r.readUint(8), maxLength: 3),
        [0x61, 0x61, 0x61],
      );
      // without a bound nothing changes
      expect(BitReader(_counted(5, 4)).readAStr(4), 'aaaaa');
    });

    test('count M + 1 is invalidValue', () {
      expect(
        () => BitReader(_counted(4, 4)).readAStr(4, maxLength: 3),
        _invalid,
      );
      expect(
        () => BitReader(_counted(4, 4)).readBinary(4, maxLength: 3),
        _invalid,
      );
      expect(
        () => BitReader(_counted(4, 4))
            .readDynArray(4, 8, (r) => r.readUint(8), maxLength: 3),
        _invalid,
      );
    });

    test('the bound is checked before the count check', () {
      // count 1000 > M and far more than the input: invalidValue, not
      // unexpectedEnd
      final short = _counted(1000, 4, bytes: 1);
      expect(() => BitReader(short).readAStr(4, maxLength: 3), _invalid);
      expect(() => BitReader(short).readBinary(4, maxLength: 3), _invalid);
      expect(
        () => BitReader(short)
            .readDynArray(4, 1, (r) => r.readBool(), maxLength: 3),
        _invalid,
      );
      // within the bound but too short: unexpectedEnd
      final short2 = _counted(3, 4, bytes: 1);
      expect(() => BitReader(short2).readAStr(4, maxLength: 3), _end);
      expect(() => BitReader(short2).readBinary(4, maxLength: 8), _end);
    });

    test('invalid bound is an ArgumentError', () {
      expect(
        () => BitReader(_counted(0, 4)).readAStr(4, maxLength: 0),
        throwsArgumentError,
      );
    });
  });

  group('encode', () {
    test('at the bound', () {
      final w = BitWriter()
        ..writeAStr('aaa', 4, maxLength: 3)
        ..writeBinary(Uint8List.fromList([1, 2, 3]), 4, maxLength: 3)
        ..writeDynArray(
          [true, false],
          4,
          (w, v) => w.writeBool(v),
          maxLength: 2,
        );
      final r = BitReader(w.toBytes());
      expect(r.readAStr(4, maxLength: 3), 'aaa');
      expect(r.readBinary(4, maxLength: 3), [1, 2, 3]);
      expect(r.readDynArray(4, 1, (r) => r.readBool(), maxLength: 2), [
        true,
        false,
      ]);
    });

    test('too long is valueOutOfRange and writes nothing', () {
      final w = BitWriter();
      expect(() => w.writeAStr('aaaa', 4, maxLength: 3), throwsEncode);
      expect(() => w.writeBinary([1, 2, 3, 4], 4, maxLength: 3), throwsEncode);
      expect(
        () => w.writeDynArray(
          [1, 2, 3],
          4,
          (w, v) => w.writeUint(v, 2),
          maxLength: 2,
        ),
        throwsEncode,
      );
      expect(w.toBytes(), isEmpty);
      // the kind
      try {
        w.writeAStr('aaaa', 4, maxLength: 3);
        fail('expected an exception');
      } on BitisEncodeException catch (e) {
        expect(e.kind, BitisEncodeErrorKind.valueOutOfRange);
      }
    });
  });

  group('MaxBits', () {
    test('dyn', () {
      expect(MaxBits.dyn(0, 32, 3), 1);
      expect(MaxBits.dyn(10, 32, 3), 9); // 1 010 1 100 0
      expect(MaxBits.dyn(7, 3, 3), 4); // no continuation bit at X
      expect(MaxBits.dyn(32, 32, 4), 11);
      // dyn agrees with the writer
      for (final (v, x, y) in [
        (0, 8, 3),
        (1, 8, 3),
        (255, 8, 3),
        (65535, 32, 4),
        (4294967295, 32, 7),
        (12345, 16, 5),
        (1, 1, 1),
      ]) {
        final w = BitWriter()..writeDynUint(v, x, y);
        expect(MaxBits.dyn(v, x, y), w.bitLength, reason: '$v $x $y');
      }
      expect(() => MaxBits.dyn(256, 8, 3), throwsArgumentError);
      expect(() => MaxBits.dyn(1, 8, 9), throwsArgumentError);
    });

    test('scalars', () {
      expect(MaxBits.boolean, 1);
      expect(MaxBits.uint(7), 7);
      expect(MaxBits.sint(7), 8);
      expect(MaxBits.dynUint(32, 4), 40);
      expect(MaxBits.dynUint(64, 64), 65);
      expect(MaxBits.dynUint(64, 7), 74);
      expect(MaxBits.dynInt(16, 5), 1 + 16 + 4);
      expect(MaxBits.float32, 32);
      expect(MaxBits.float64, 64);
      expect(MaxBits.fixedPoint(10), 10);
    });

    test('containers', () {
      expect(MaxBits.astr(4, null), isNull);
      expect(MaxBits.astr(4, 32), MaxBits.dyn(32, 32, 4) + 256);
      expect(MaxBits.binary(8, 1), MaxBits.dyn(1, 32, 8) + 8);
      expect(MaxBits.dynArray(2, 3, 11), MaxBits.dyn(3, 32, 2) + 33);
      expect(MaxBits.dynArray(2, 3, null), isNull);
      expect(MaxBits.dynArray(2, null, 11), isNull);
      expect(MaxBits.optional(5), 6);
      expect(MaxBits.optional(null), isNull);
      expect(MaxBits.fixedArray(3, 5), 15);
      expect(MaxBits.fixedArray(3, null), isNull);
      expect(MaxBits.enumeration(1, 3), 1);
      expect(MaxBits.enumeration(5, 2), MaxBits.dyn(4, 32, 2));
      expect(MaxBits.oneOf(2, [1, 11, 41]), MaxBits.dyn(2, 32, 2) + 41);
      expect(MaxBits.oneOf(2, [1, null]), isNull);
      expect(MaxBits.message([]), 0);
      expect(MaxBits.message([1, 2, 3]), 6);
      expect(MaxBits.message([1, null]), isNull);
    });

    test('results above 2^53 - 1 are null', () {
      expect(MaxBits.add(MaxBits.maxSafe, 0), MaxBits.maxSafe);
      expect(MaxBits.add(MaxBits.maxSafe, 1), isNull);
      expect(MaxBits.mul(1 << 30, 1 << 23), isNull);
      expect(MaxBits.mul(1 << 30, 1 << 22), 4503599627370496);
      final big = MaxBits.dynArray(4, 65535, 65535 * 8 * 65535)!;
      expect(MaxBits.dynArray(4, 65535, big), isNull);
    });
  });
}
