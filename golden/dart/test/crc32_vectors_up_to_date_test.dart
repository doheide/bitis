// The embedded CRC vectors (crc32_vectors.g.dart) must be the current
// golden/vectors/crc32.txt. VM only (file I/O); the web runs use the embedded
// copy checked here.
@TestOn('vm')
library;

import 'dart:io';

import 'package:test/test.dart';

import 'crc32_vectors.g.dart';

void main() {
  test('test/crc32_vectors.g.dart is up to date', () {
    // dart test runs with the package directory (golden/dart) as cwd
    expect(
      File('../vectors/crc32.txt').readAsStringSync() == crc32Vectors,
      isTrue,
      reason: 'run: python3 golden/vectors/crc32_gen.py',
    );
  });
}
