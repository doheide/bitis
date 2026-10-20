// The embedded manifest (manifest.g.dart) must be the current
// golden/vectors/manifest.txt. VM only (file I/O); the web runs use the
// embedded copy checked here.
@TestOn('vm')
library;

import 'dart:io';

import 'package:test/test.dart';

import '../tool/embed_manifest.dart' as embed;

void main() {
  test('test/manifest.g.dart is up to date', () {
    // dart test runs with the package directory (golden/dart) as cwd
    final manifest = File('../vectors/manifest.txt').readAsStringSync();
    expect(
      File('test/manifest.g.dart').readAsStringSync() == embed.render(manifest),
      isTrue,
      reason: 'run: dart run tool/embed_manifest.dart',
    );
  });
}
