// CRC-32 helper of package:bitis against the shared vectors
// golden/vectors/crc32.txt (embedded as crc32_vectors.g.dart, written by
// golden/vectors/crc32_gen.py): one-shot, incremental with several splits,
// byte by byte, reset.
import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

import 'crc32_vectors.g.dart';

/// One record of crc32.txt.
class CrcVector {
  /// Creates a record.
  CrcVector(this.name, this.data, this.crc);

  /// Name of the vector.
  final String name;

  /// Input bytes.
  final List<int> data;

  /// Expected CRC.
  final int crc;
}

List<int> _unhex(String s) {
  if (s == '-') return const [];
  if (s.length.isOdd) throw FormatException('odd hex length', s);
  return [
    for (var i = 0; i < s.length; i += 2)
      int.parse(s.substring(i, i + 2), radix: 16),
  ];
}

/// Parses the text of crc32.txt.
List<CrcVector> parseCrcVectors(String text) {
  final out = <CrcVector>[];
  for (final line in text.split('\n')) {
    if (line.isEmpty || line.startsWith('#')) continue;
    final f = line.split(' ');
    if (f.length != 4) throw FormatException('bad line', line);
    final data = switch (f[0]) {
      'hex' => _unhex(f[2]),
      'ascii' => f[2].codeUnits,
      _ => throw FormatException('unknown record kind', line),
    };
    if (f[3].length != 8) throw FormatException('bad crc', line);
    out.add(CrcVector(f[1], data, int.parse(f[3], radix: 16)));
  }
  return out;
}

void main() {
  final vectors = parseCrcVectors(crc32Vectors);

  test('vector file', () {
    expect(vectors.length, greaterThanOrEqualTo(16));
    final check = vectors.singleWhere((v) => v.name == 'check');
    expect(check.data, '123456789'.codeUnits);
    expect(check.crc, 0xCBF43926);
  });

  for (final v in vectors) {
    test('crc32 ${v.name}', () {
      final d = v.data;
      final n = d.length;
      expect(crc32(d), v.crc, reason: 'one-shot');

      // splits at the start, the end, the middle and every position of short
      // inputs
      final cuts = {0, 1, n ~/ 2, n > 0 ? n - 1 : 0, n};
      if (n <= 64) cuts.addAll(List<int>.generate(n + 1, (i) => i));
      for (final i in cuts.where((i) => i <= n)) {
        final c =
            Crc32()
              ..update(d.sublist(0, i))
              ..update(d.sublist(i));
        expect(c.finish(), v.crc, reason: 'split at $i');
      }
      // three pieces, with empty updates in between
      final a = n ~/ 3, b = 2 * n ~/ 3;
      final c3 =
          Crc32()
            ..update(d.sublist(0, a))
            ..update(const [])
            ..update(d.sublist(a, b))
            ..update(d.sublist(b))
            ..update(const []);
      expect(c3.value, v.crc, reason: 'three pieces');
      // byte by byte
      final c = Crc32();
      for (var i = 0; i < n; i++) {
        c.update([d[i]]);
        if (i == n ~/ 2) expect(c.value, crc32(d.sublist(0, i + 1)));
      }
      expect(c.finish(), v.crc, reason: 'byte by byte');
      c.reset();
      expect(c.value, 0);
      c.update(d);
      expect(c.value, v.crc, reason: 'after reset');
    });
  }
}
