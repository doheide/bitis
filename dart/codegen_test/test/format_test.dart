// The generated code is formatted like `dart format` (tall style, Dart 3.7+):
// without its `dart format off` marker the formatter does not change it.
@TestOn('vm')
library;

import 'dart:io';

import 'package:test/test.dart';

void main() {
  final files =
      Directory('lib')
          .listSync()
          .whereType<File>()
          .where((f) => f.path.endsWith('.dart'))
          .toList()
        ..sort((a, b) => a.path.compareTo(b.path));

  test('generated files exist', () => expect(files, hasLength(4)));

  for (final file in files) {
    test('${file.path} is formatted', () async {
      final source = file.readAsStringSync();
      expect(source, contains('\n// dart format off\n'));
      final tmp = Directory.systemTemp.createTempSync('bitis_fmt');
      addTearDown(() => tmp.deleteSync(recursive: true));
      final copy = File(
        '${tmp.path}/${file.uri.pathSegments.last}',
      )..writeAsStringSync(source.replaceFirst('\n// dart format off\n', '\n'));
      final original = copy.readAsStringSync();
      final r = await Process.run(Platform.resolvedExecutable, [
        'format',
        '--language-version=3.7',
        copy.path,
      ]);
      expect(r.exitCode, 0, reason: '${r.stderr}');
      expect(copy.readAsStringSync(), original);
    });
  }
}
