// The embedded CRC-8/16 vectors (crc8_16_vectors.g.dart) must be the current
// golden/vectors/crc8_16.txt. VM only (file I/O); the web runs use the embedded
// copy checked here.
@TestOn('vm')
library;

import 'dart:io';

import 'package:test/test.dart';

import 'crc8_16_vectors.g.dart';

void main() {
  test('test/crc8_16_vectors.g.dart is up to date', () {
    // dart test runs with the package directory (golden/dart) as cwd
    expect(
      File('../vectors/crc8_16.txt').readAsStringSync() == crc8Crc16Vectors,
      isTrue,
      reason: 'run: python3 golden/vectors/crc8_16_gen.py',
    );
  });
}
