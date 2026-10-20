// Generated code for maximum lengths `[M]` (spec 2.16) and bitisMaxBits.
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:bitis_codegen_test/bounds.dart';
import 'package:test/test.dart';

Matcher _decodeError(BitisDecodeErrorKind kind) =>
    throwsA(isA<BitisDecodeException>().having((e) => e.kind, 'kind', kind));
final _invalid = _decodeError(BitisDecodeErrorKind.invalidValue);
final _end = _decodeError(BitisDecodeErrorKind.unexpectedEnd);
final _outOfRange = throwsA(
  isA<BitisEncodeException>().having(
    (e) => e.kind,
    'kind',
    BitisEncodeErrorKind.valueOutOfRange,
  ),
);

Uint8List _bytes(int n) => Uint8List.fromList(List.filled(n, 7));

/// A [Bounded] with every bounded field exactly at its maximum length.
Bounded _full() => Bounded(
  name: 'abcde',
  data: _bytes(4),
  tags: ['ab', 'cd', 'ef'],
  note: 'abcdef',
  pair: [_bytes(2), _bytes(2)],
  items: [Item(a: 15, tag: 'xyz'), Item(a: 1, tag: 'uvw')],
  choice: const OO_Bounded_Choice_Text('wxyz'),
);

void main() {
  test('bitisMaxBits (spec section 4)', () {
    final item = MaxBits.message([MaxBits.uint(4), MaxBits.astr(2, 3)]);
    expect(Item.bitisMaxBits, item);
    expect(
      Bounded.bitisMaxBits,
      MaxBits.message([
        MaxBits.astr(4, 5),
        MaxBits.binary(3, 4),
        MaxBits.dynArray(2, 3, MaxBits.astr(4, 2)),
        MaxBits.optional(MaxBits.astr(4, 6)),
        MaxBits.fixedArray(2, MaxBits.binary(2, 2)),
        MaxBits.dynArray(1, 2, item),
        MaxBits.oneOf(1, [MaxBits.astr(3, 4), MaxBits.enumeration(3, 2)]),
      ]),
    );
    expect(Bounded.bitisMaxBits, 357);
    expect(Unbounded.bitisMaxBits, isNull);
  });

  test('values at the bound round trip; the size is at most maxBits', () {
    final m = _full();
    final enc = bitisEncodeWithSize(m);
    expect(enc.totalBits, lessThanOrEqualTo(Bounded.bitisMaxBits));
    expect(Bounded.fromBytesExact(enc.bytes), equals(m));
  });

  test('encoding a too long value is valueOutOfRange', () {
    for (final m in [
      _full().copyWith(name: 'abcdef'),
      _full().copyWith(data: _bytes(5)),
      _full().copyWith(tags: ['a', 'b', 'c', 'd']),
      _full().copyWith(tags: ['abc']),
      _full().copyWith(note: 'abcdefg'),
      _full().copyWith(pair: [_bytes(3), _bytes(2)]),
      _full().copyWith(items: [Item(), Item(), Item()]),
      _full().copyWith(items: [Item(tag: 'abcd')]),
      _full().copyWith(choice: const OO_Bounded_Choice_Text('abcde')),
    ]) {
      expect(() => m.toBytes(), _outOfRange, reason: '$m');
    }
  });

  test('decoding a count M + 1 is invalidValue, also before unexpectedEnd', () {
    // name: astr_d4[5] with count 6 (then the input ends): uint_32d4 6 =
    // 1 0110 0 -> invalidValue, not unexpectedEnd
    final w = BitWriter()..writeDynUint(6, 32, 4);
    expect(() => Bounded.fromBytes(w.toBytes()), _invalid);
    // count 5 but no data: unexpectedEnd
    final w2 = BitWriter()..writeDynUint(5, 32, 4);
    expect(() => Bounded.fromBytes(w2.toBytes()), _end);

    // an element of a bounded repeated_dyn: tags[0] with 3 bytes
    final w3 =
        BitWriter()
          ..writeAStr('a', 4)
          ..writeBinary(Uint8List(0), 3)
          ..writeDynUint(1, 32, 2)
          ..writeAStr('abc', 4);
    expect(() => Bounded.fromBytes(w3.toBytes()), _invalid);

    // too many elements (4 > 3) of tags with complete data
    final w4 =
        BitWriter()
          ..writeAStr('a', 4)
          ..writeBinary(Uint8List(0), 3)
          ..writeDynArray(['a', 'b', 'c', 'd'], 2, (w, v) => w.writeAStr(v, 4));
    expect(() => Bounded.fromBytes(w4.toBytes()), _invalid);
  });
}
