// CRC-8/AUTOSAR, CRC-16/IBM-3740 and transmission CRC helpers of package:bitis
// against the shared vectors golden/vectors/crc8_16.txt (embedded as
// crc8_16_vectors.g.dart, written by golden/vectors/crc8_16_gen.py): one-shot,
// incremental with several splits, byte by byte, reset; frames with
// append / verify / strip (little-endian CRC-16 trailer).
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

import 'crc8_16_vectors.g.dart';

/// A `hex` / `ascii` record of crc8_16.txt.
class Crc816Vector {
  /// Creates a record.
  Crc816Vector(this.name, this.data, this.crc8, this.crc16);

  /// Name of the vector.
  final String name;

  /// Input bytes.
  final List<int> data;

  /// Expected CRC-8/AUTOSAR.
  final int crc8;

  /// Expected CRC-16/IBM-3740.
  final int crc16;
}

/// A `frame` record of crc8_16.txt.
class Crc816Frame {
  /// Creates a record.
  Crc816Frame(this.name, this.header, this.payload, this.trailer);

  /// Name of the frame.
  final String name;

  /// Header bytes.
  final List<int> header;

  /// Payload bytes.
  final List<int> payload;

  /// Expected trailer (1 or 2 bytes).
  final List<int> trailer;
}

List<int> _unhex(String s) {
  if (s == '-') return const [];
  if (s.length.isOdd) throw FormatException('odd hex length', s);
  return [
    for (var i = 0; i < s.length; i += 2)
      int.parse(s.substring(i, i + 2), radix: 16),
  ];
}

void main() {
  final vectors = <Crc816Vector>[];
  final frames = <Crc816Frame>[];
  for (final line in crc8Crc16Vectors.split('\n')) {
    if (line.isEmpty || line.startsWith('#')) continue;
    final f = line.split(' ');
    if (f.length != 5) throw FormatException('bad line', line);
    if (f[0] == 'frame') {
      frames.add(Crc816Frame(f[1], _unhex(f[2]), _unhex(f[3]), _unhex(f[4])));
      continue;
    }
    final data = switch (f[0]) {
      'hex' => _unhex(f[2]),
      'ascii' => f[2].codeUnits,
      _ => throw FormatException('unknown record kind', line),
    };
    if (f[3].length != 2 || f[4].length != 4) {
      throw FormatException('bad crc', line);
    }
    vectors.add(
      Crc816Vector(
        f[1],
        data,
        int.parse(f[3], radix: 16),
        int.parse(f[4], radix: 16),
      ),
    );
  }

  test('vector file', () {
    expect(vectors.length, greaterThanOrEqualTo(16));
    expect(frames.length, greaterThanOrEqualTo(11));
    final check = vectors.singleWhere((v) => v.name == 'check');
    expect(check.data, '123456789'.codeUnits);
    expect([check.crc8, check.crc16], [0xDF, 0x29B1]);
    final ns = {for (final f in frames) f.header.length + f.payload.length};
    expect(ns, containsAll([1, 14, 15]));
  });

  for (final v in vectors) {
    test('crc8/crc16 ${v.name}', () {
      final d = v.data;
      final n = d.length;
      expect(crc8Autosar(d), v.crc8, reason: 'one-shot');
      expect(crc16Ibm3740(d), v.crc16, reason: 'one-shot');

      final cuts = {0, 1, n ~/ 2, n > 0 ? n - 1 : 0, n};
      if (n <= 64) cuts.addAll(List<int>.generate(n + 1, (i) => i));
      for (final i in cuts.where((i) => i <= n)) {
        final c8 = Crc8Autosar()
          ..update(d.sublist(0, i))
          ..update(d.sublist(i));
        final c16 = Crc16Ibm3740()
          ..update(d.sublist(0, i))
          ..update(d.sublist(i));
        expect(
          [c8.finish(), c16.finish()],
          [v.crc8, v.crc16],
          reason: 'split at $i',
        );
      }
      final a = n ~/ 3, b = 2 * n ~/ 3;
      final c8 = Crc8Autosar()
        ..update(d.sublist(0, a))
        ..update(const [])
        ..update(d.sublist(a, b))
        ..update(d.sublist(b))
        ..update(const []);
      final c16 = Crc16Ibm3740()
        ..update(d.sublist(0, a))
        ..update(const [])
        ..update(d.sublist(a, b))
        ..update(d.sublist(b))
        ..update(const []);
      expect([c8.value, c16.value], [v.crc8, v.crc16], reason: 'three pieces');
      c8.reset();
      c16.reset();
      for (var i = 0; i < n; i++) {
        c8.update([d[i]]);
        c16.update([d[i]]);
        if (i == n ~/ 2) {
          expect(c8.value, crc8Autosar(d.sublist(0, i + 1)));
          expect(c16.value, crc16Ibm3740(d.sublist(0, i + 1)));
        }
      }
      expect([c8.value, c16.value], [v.crc8, v.crc16], reason: 'byte by byte');
      // property of the CRC function itself (not used by the transmission
      // rule, whose trailer is little-endian): CRC over data + CRC big-endian
      expect(
        crc16Ibm3740([...d, v.crc16 >>> 8, v.crc16 & 0xFF]),
        0,
        reason: 'residue',
      );
    });
  }

  for (final f in frames) {
    test('transmission frame ${f.name}', () {
      final frame = [...f.header, ...f.payload];
      final n = frame.length;
      for (var i = 0; i < f.payload.length; i++) {
        expect(f.payload[i], (i * 37 + 1) & 0xFF);
      }
      expect(transmissionCrcLen(n), f.trailer.length);
      final crc = transmissionCrc(frame);
      expect(
        n <= 14 ? [crc] : [crc & 0xFF, crc >>> 8],
        f.trailer,
        reason: 'transmissionCrc',
      );
      final full = appendTransmissionCrc(frame);
      expect(full, [...frame, ...f.trailer]);
      expect(verifyTransmissionCrc(full), isTrue);
      expect(stripTransmissionCrc(full), n);
      for (var bit = 0; bit < full.length * 8; bit++) {
        final bad = Uint8List.fromList(full);
        bad[bit >> 3] ^= 1 << (bit & 7);
        expect(stripTransmissionCrc(bad), isNull, reason: 'bit $bit');
      }
    });
  }
}
