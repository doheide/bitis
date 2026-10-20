import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

void main() {
  group('BitisEq', () {
    const fp = FixedPointType(8, 0, 10);

    test('listEquals / listHash', () {
      expect(BitisEq.listEquals([1, 2], [1, 2]), isTrue);
      expect(BitisEq.listEquals([1, 2], [1, 3]), isFalse);
      expect(BitisEq.listEquals([1, 2], [1, 2, 3]), isFalse);
      expect(BitisEq.listEquals<int>([], []), isTrue);
      expect(
        BitisEq.listEquals(Uint8List.fromList([1, 2]), [1, 2]),
        isTrue,
      );
      expect(BitisEq.listHash([1, 2]),
          BitisEq.listHash(Uint8List.fromList([1, 2])));
      final a = [
        Uint8List.fromList([1]),
        Uint8List.fromList([2, 3])
      ];
      final b = [
        Uint8List.fromList([1]),
        Uint8List.fromList([2, 3])
      ];
      expect(BitisEq.listEquals(a, b), isFalse); // identity of the elements
      expect(BitisEq.listEquals(a, b, BitisEq.listEquals<int>), isTrue);
      expect(BitisEq.listHash(a, Object.hashAll),
          BitisEq.listHash(b, Object.hashAll));
    });

    test('fixed point elements are compared on the raw value', () {
      final a = [const FixedPoint.value(5.0), const FixedPoint.value(11)];
      final b = [const FixedPoint.value(5.001), const FixedPoint.overflow()];
      expect(BitisEq.listEquals(a, b), isFalse);
      expect(BitisEq.listEquals(a, b, fp.sameEncoding), isTrue);
      expect(
          BitisEq.listHash(a, fp.encodeRaw), BitisEq.listHash(b, fp.encodeRaw));
    });

    test('optionalEquals / optionalHash', () {
      expect(BitisEq.optionalEquals<FixedPoint>(null, null, fp.sameEncoding),
          isTrue);
      expect(
          BitisEq.optionalEquals<FixedPoint>(
              null, const FixedPoint.value(1), fp.sameEncoding),
          isFalse);
      expect(
          BitisEq.optionalEquals<FixedPoint>(
              const FixedPoint.value(1), null, fp.sameEncoding),
          isFalse);
      expect(
        BitisEq.optionalEquals(const FixedPoint.value(1),
            const FixedPoint.value(1.001), fp.sameEncoding),
        isTrue,
      );
      expect(BitisEq.optionalHash<FixedPoint>(null, fp.encodeRaw), isNull);
      expect(
          BitisEq.optionalHash(const FixedPoint.underflow(), fp.encodeRaw), 0);
    });
  });

  test('bitisKeepOr', () {
    expect(bitisKeepOr<int>(bitisKeep, 3), 3);
    expect(bitisKeepOr<int>(null, 3), isNull);
    expect(bitisKeepOr<int>(5, 3), 5);
    expect(() => bitisKeepOr<int>('x', 3), throwsA(isA<TypeError>()));
    expect(bitisKeep.toString(), 'bitisKeep');
  });

  group('BitisJson list / optional helpers', () {
    test('optionalFromJson', () {
      expect(BitisJson.optionalFromJson(null, BitisJson.intFromJson), isNull);
      expect(BitisJson.optionalFromJson(7, BitisJson.intFromJson), 7);
      expect(
        () => BitisJson.optionalFromJson('x', BitisJson.boolFromJson),
        throwsFormatException,
      );
    });

    test('listFromJsonWith / listToJson', () {
      expect(
          BitisJson.listFromJsonWith([1, '2'], BitisJson.intFromJson), [1, 2]);
      expect(BitisJson.listFromJsonWith([true], BitisJson.boolFromJson, 1),
          [true]);
      expect(
        () => BitisJson.listFromJsonWith([1], BitisJson.intFromJson, 2),
        throwsFormatException,
      );
      expect(() => BitisJson.listFromJsonWith(1, BitisJson.intFromJson),
          throwsFormatException);
      expect(
          BitisJson.listToJson(
              [Int64(1), Int64.MAX_VALUE], BitisJson.int64ToJson),
          [1, '9223372036854775807']);
    });
  });
}
