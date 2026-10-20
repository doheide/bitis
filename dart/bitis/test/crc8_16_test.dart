import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

/// Bit-wise CRC-8/AUTOSAR reference without table.
int crc8Bitwise(List<int> data) {
  var c = 0xFF;
  for (final b in data) {
    c ^= b & 0xFF;
    for (var k = 0; k < 8; k++) {
      c = (c & 0x80) != 0 ? ((c << 1) ^ 0x2F) & 0xFF : (c << 1) & 0xFF;
    }
  }
  return c ^ 0xFF;
}

/// Bit-wise CRC-16/IBM-3740 reference without table.
int crc16Bitwise(List<int> data) {
  var c = 0xFFFF;
  for (final b in data) {
    c ^= (b & 0xFF) << 8;
    for (var k = 0; k < 8; k++) {
      c = (c & 0x8000) != 0 ? ((c << 1) ^ 0x1021) & 0xFFFF : (c << 1) & 0xFFFF;
    }
  }
  return c;
}

void main() {
  test('check values, empty input', () {
    expect(crc8Autosar('123456789'.codeUnits), 0xDF);
    expect(crc16Ibm3740('123456789'.codeUnits), 0x29B1);
    expect(crc8Autosar(Uint8List.fromList('123456789'.codeUnits)), 0xDF);
    expect(crc16Ibm3740(Uint8List.fromList('123456789'.codeUnits)), 0x29B1);
    expect(crc8Autosar(const []), 0);
    expect(crc16Ibm3740(const []), 0xFFFF);
    expect(Crc8Autosar().value, 0);
    expect(Crc16Ibm3740().finish(), 0xFFFF);
  });

  test('matches the bit-wise reference, results unsigned', () {
    final data = List<int>.generate(2100, (i) => (i * 167 + (i >> 3)) & 0xFF);
    for (var n = 0; n < 300; n++) {
      final d = data.sublist(0, n);
      expect(crc8Autosar(d), allOf(inInclusiveRange(0, 0xFF), crc8Bitwise(d)));
      expect(
        crc16Ibm3740(d),
        allOf(inInclusiveRange(0, 0xFFFF), crc16Bitwise(d)),
      );
    }
    expect(crc16Ibm3740(data), crc16Bitwise(data));
  });

  test('only the low 8 bits of each element are used', () {
    expect(
      crc8Autosar([0x131, -0xCF, 0x1033]),
      crc8Autosar([0x31, 0x31, 0x33]),
    );
    expect(
      crc16Ibm3740([0x131, -0xCF, 0x1033]),
      crc16Ibm3740([0x31, 0x31, 0x33]),
    );
    final f = List<int>.generate(20, (i) => i + 0x100);
    expect(
      appendTransmissionCrc(f),
      appendTransmissionCrc([for (final b in f) b & 0xFF]),
    );
    expect(verifyTransmissionCrc([0x102, 0x101, 0x195]), isTrue);
  });

  test('incremental: every split, finish does not change the state', () {
    final data = 'The quick brown fox jumps over the lazy dog'.codeUnits;
    final e8 = crc8Autosar(data);
    final e16 = crc16Ibm3740(data);
    for (var i = 0; i <= data.length; i++) {
      for (var j = i; j <= data.length; j++) {
        final c8 = Crc8Autosar()..update(data.sublist(0, i));
        final c16 = Crc16Ibm3740()..update(data.sublist(0, i));
        c8.update(data.sublist(i, j));
        c16.update(data.sublist(i, j));
        expect(c8.finish(), crc8Autosar(data.sublist(0, j)));
        expect(c16.finish(), crc16Ibm3740(data.sublist(0, j)));
        c8.update(data.sublist(j));
        c16.update(data.sublist(j));
        expect([c8.value, c16.value], [e8, e16], reason: 'split $i, $j');
      }
    }
  });

  test('reset', () {
    final c8 = Crc8Autosar()..update([1, 2, 3]);
    final c16 = Crc16Ibm3740()..update([1, 2, 3]);
    c8.reset();
    c16.reset();
    expect([c8.value, c16.value], [0, 0xFFFF]);
    c8.update('123456789'.codeUnits);
    c16.update('123456789'.codeUnits);
    expect([c8.value, c16.value], [0xDF, 0x29B1]);
  });

  test('transmission CRC: boundaries, little-endian, single-bit errors', () {
    expect(transmissionCrcLen(0), 1);
    expect(transmissionCrcLen(14), 1);
    expect(transmissionCrcLen(15), 2);
    // n >= 1: real frames always have a header (L = 1 is rejected, see below)
    for (var n = 1; n < 40; n++) {
      final frame = List<int>.generate(n, (i) => (i * 37 + 1) & 0xFF);
      final full = appendTransmissionCrc(frame);
      expect(full.length, n + transmissionCrcLen(n));
      expect(full.length, isNot(16));
      expect(full.sublist(0, n), frame);
      expect(stripTransmissionCrc(full), n);
      expect(verifyTransmissionCrc(full), isTrue);
      if (n >= 15) {
        final c = crc16Ibm3740(frame);
        expect(full.sublist(n), [c & 0xFF, c >>> 8], reason: 'little-endian');
      }
      for (var bit = 0; bit < full.length * 8; bit++) {
        final bad = Uint8List.fromList(full);
        bad[bit >> 3] ^= 1 << (bit & 7);
        expect(verifyTransmissionCrc(bad), isFalse, reason: 'n $n bit $bit');
      }
    }
    // L = 16 is invalid, also with a matching CRC-16 over 14 bytes
    final l16 = List<int>.filled(14, 7, growable: true);
    final c = crc16Ibm3740(l16);
    l16.addAll([c & 0xFF, c >>> 8]);
    expect(stripTransmissionCrc(l16), isNull);
    final l16b = List<int>.filled(15, 7, growable: true)
      ..add(crc8Autosar(List.filled(15, 7)));
    expect(verifyTransmissionCrc(l16b), isFalse);
    expect(verifyTransmissionCrc(const []), isFalse);
    expect(appendTransmissionCrc(const []), [0x00]);
    // L = 1 (no header + payload) is rejected: a stray 0x00 byte is not a frame
    expect(stripTransmissionCrc([0x00]), isNull);
    expect(transmissionCrc('123456789'.codeUnits), 0xDF);
    expect(
      transmissionCrc(List.filled(15, 0)),
      crc16Ibm3740(List.filled(15, 0)),
    );
  });
}
