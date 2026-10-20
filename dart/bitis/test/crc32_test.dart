import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

/// Bit-wise reference without table, using arithmetic only where the web
/// could differ (no `~`, no `<<`).
int crc32Bitwise(List<int> data) {
  var c = 0xFFFFFFFF;
  for (final b in data) {
    c ^= b & 0xFF;
    for (var k = 0; k < 8; k++) {
      c = (c & 1) != 0 ? (c >>> 1) ^ 0xEDB88320 : c >>> 1;
    }
  }
  return c ^ 0xFFFFFFFF;
}

void main() {
  test('check value, empty input', () {
    expect(crc32('123456789'.codeUnits), 0xCBF43926);
    expect(crc32(Uint8List.fromList('123456789'.codeUnits)), 0xCBF43926);
    expect(crc32(const []), 0);
    expect(Crc32().value, 0);
    expect(Crc32().finish(), 0);
  });

  test('results are unsigned 32-bit (also with the top bit set)', () {
    // 0xff000000, 0xffffffff, 0xcbf43926 have bit 31 set
    expect(crc32([0xFF]), 0xFF000000);
    expect(crc32([0xFF, 0xFF, 0xFF, 0xFF]), 0xFFFFFFFF);
    final data = List<int>.generate(4096, (i) => (i * 167 + (i >> 3)) & 0xFF);
    for (var n = 0; n < 300; n++) {
      final c = crc32(data.sublist(0, n));
      expect(c, inInclusiveRange(0, 0xFFFFFFFF));
      expect(c, crc32Bitwise(data.sublist(0, n)), reason: 'n = $n');
    }
    expect(crc32(data), crc32Bitwise(data));
  });

  test('only the low 8 bits of each element are used', () {
    expect(crc32([0x131, -0xCF, 0x1033]), crc32([0x31, 0x31, 0x33]));
  });

  test('incremental: every split, finish does not change the state', () {
    final data = 'The quick brown fox jumps over the lazy dog'.codeUnits;
    final expected = crc32(data);
    expect(expected, 0x414FA339);
    for (var i = 0; i <= data.length; i++) {
      for (var j = i; j <= data.length; j++) {
        final c = Crc32()..update(data.sublist(0, i));
        c.update(data.sublist(i, j));
        expect(c.finish(), crc32(data.sublist(0, j)));
        c.update(data.sublist(j));
        expect(c.value, expected, reason: 'split $i, $j');
      }
    }
  });

  test('reset', () {
    final c = Crc32()..update([1, 2, 3]);
    c.reset();
    expect(c.value, 0);
    c.update('123456789'.codeUnits);
    expect(c.value, 0xCBF43926);
  });
}
