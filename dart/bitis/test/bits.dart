// Bit string helpers: a bit string lists stream bits in order (stream bit 0
// first), spaces are ignored. This is the notation of docs/WIRE_FORMAT.md
// (e.g. `10` as uint_32d3 is `1 010 1 100 0`).
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

/// Removes spaces.
String clean(String bits) => bits.replaceAll(' ', '');

/// Bytes for a bit string (zero padded).
Uint8List bitsToBytes(String bits) {
  final s = clean(bits);
  final out = Uint8List((s.length + 7) ~/ 8);
  for (var i = 0; i < s.length; i++) {
    if (s[i] == '1') {
      out[i ~/ 8] |= 1 << (i % 8);
    } else if (s[i] != '0') {
      throw ArgumentError('bad bit ${s[i]}');
    }
  }
  return out;
}

/// The low [width] bits of [v] in stream order (LSB first), computed with
/// BigInt independently of the runtime.
String lsb(Object v, int width) {
  final b = switch (v) {
    final int i => BigInt.from(i),
    final Int64 i => BigInt.parse(i.toStringUnsigned()),
    final BigInt b => b,
    _ => throw ArgumentError(v),
  };
  final sb = StringBuffer();
  for (var k = 0; k < width; k++) {
    sb.write(((b >> k) & BigInt.one) == BigInt.one ? '1' : '0');
  }
  return sb.toString();
}

/// Expects that [write] produces exactly [bits].
void expectWrites(void Function(BitWriter w) write, String bits) {
  final w = BitWriter();
  write(w);
  expect(w.bitLength, clean(bits).length, reason: 'bit length');
  expect(w.toBytes(), bitsToBytes(bits));
}

/// Reads [bits] with [read] and expects that exactly all bits are consumed.
T readsAll<T>(String bits, T Function(BitReader r) read) {
  final r = BitReader(bitsToBytes(bits));
  final v = read(r);
  expect(r.bitPosition, clean(bits).length, reason: 'consumed bits');
  return v;
}

/// Matcher for a [BitisDecodeException] of [kind].
Matcher throwsDecode(BitisDecodeErrorKind kind) => throwsA(
      isA<BitisDecodeException>().having((e) => e.kind, 'kind', kind),
    );

/// Matcher for a [BitisEncodeException] (valueOutOfRange).
final Matcher throwsEncode = throwsA(
  isA<BitisEncodeException>().having(
    (e) => e.kind,
    'kind',
    BitisEncodeErrorKind.valueOutOfRange,
  ),
);
