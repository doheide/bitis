// Golden vector tests of the Dart runtime (see golden/README.md).
//
// Checks every record of golden/vectors/manifest.txt (embedded in
// manifest.g.dart), exactly like tests/golden.rs and golden/cpp/golden_test.cpp:
// - valid:    the default and the exact decoder consume total_bits, re-encoding
//             gives the same bytes; with appended bytes the default decoder
//             still succeeds, the exact one fails with trailingData,
// - noncanon: decoding succeeds with total_bits, re-encoding gives the
//             canonical bytes,
// - invalid:  decoding fails with the given BitisDecodeErrorKind,
// - min_bits: bitisMinBits of every message type,
// - max_bits: bitisMaxBits of every message type (null = unbounded), and every
//             valid encoding is within it,
// plus a JSON round trip of every valid case (decode -> toJson -> JSON text ->
// fromJson -> encode gives the same bytes).
//
// The code under test is golden/gen/dart/golden.dart, generated from
// golden/schema/golden.bitis.
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

import '../../gen/dart/golden.dart' as golden;
import 'manifest.dart';
import 'manifest.g.dart';

/// Decodes and re-encodes, or returns the decode exception.
Object _outcome(Record r, List<int> data, {required bool exact}) {
  try {
    return messageType(r.type).roundtrip(data, exact: exact);
  } on BitisDecodeException catch (e) {
    return e;
  }
}

String _describe(Object o) => switch (o) {
  final Roundtrip t =>
    'decoded (${t.bits} bits, re-encoded ${hex(t.reencoded)})',
  final BitisDecodeException e => manifestErrorName(e.kind),
  _ => '$o',
};

void main() {
  test('manifest belongs to the schema of the generated code', () {
    expect(
      goldenManifest,
      contains('schema hash ${golden.bitisSchemaHashHex}\n'),
      reason: 'manifest was generated for another schema',
    );
    expect(records(RecordKind.valid).length, greaterThan(40));
    expect(records(RecordKind.noncanon), isNotEmpty);
    expect(records(RecordKind.invalid).length, greaterThan(40));
    expect(records(RecordKind.minBits).length, 16);
    expect(records(RecordKind.maxBits).length, 16);
    expect(
      manifest.map((r) => r.name).toSet().length,
      manifest.length,
      reason: 'case names must be unique',
    );
  });

  test('error names of the manifest map to BitisDecodeErrorKind', () {
    expect(BitisDecodeErrorKind.values.map(manifestErrorName), [
      'UnexpectedEnd',
      'InvalidValue',
      'InvalidDiscriminant',
      'TrailingData',
    ]);
    for (final k in BitisDecodeErrorKind.values) {
      expect(parseErrorName(manifestErrorName(k)), k);
    }
  });

  group('valid', () {
    for (final r in records(RecordKind.valid)) {
      test(r.name, () {
        expect(
          r.data.length,
          (r.bits + 7) ~/ 8,
          reason: 'hex length / total_bits mismatch in the manifest',
        );
        for (final exact in [false, true]) {
          final o = _outcome(r, r.data, exact: exact);
          expect(o, isA<Roundtrip>(), reason: 'exact=$exact: ${_describe(o)}');
          final t = o as Roundtrip;
          expect(t.bits, r.bits, reason: 'total_bits, exact=$exact');
          expect(hex(t.reencoded), hex(r.data), reason: 'exact=$exact');
          expect(t.reencodedBits, r.bits, reason: 'encoded bits, exact=$exact');
        }
        // trailing bytes: ignored by the default decoder, rejected by the
        // exact one
        final longer = Uint8List.fromList([...r.data, 0xa5, 0xff]);
        final o = _outcome(r, longer, exact: false);
        expect(o, isA<Roundtrip>(), reason: 'trailing bytes: ${_describe(o)}');
        final t = o as Roundtrip;
        expect(t.bits, r.bits);
        expect(hex(t.reencoded), hex(r.data));
        expect(
          _describe(_outcome(r, longer, exact: true)),
          'TrailingData',
          reason: 'exact decoder with trailing bytes',
        );
      });
    }
  });

  group('noncanon', () {
    for (final r in records(RecordKind.noncanon)) {
      test(r.name, () {
        for (final exact in [false, true]) {
          final o = _outcome(r, r.data, exact: exact);
          expect(o, isA<Roundtrip>(), reason: 'exact=$exact: ${_describe(o)}');
          final t = o as Roundtrip;
          expect(t.bits, r.bits, reason: 'total_bits, exact=$exact');
          expect(hex(t.reencoded), hex(r.canonical), reason: 'exact=$exact');
        }
      });
    }
  });

  group('invalid', () {
    for (final r in records(RecordKind.invalid)) {
      test(r.name, () {
        // anything but a BitisDecodeException fails the test
        expect(
          _describe(_outcome(r, r.data, exact: r.exact)),
          manifestErrorName(r.error!),
          reason: r.exact ? 'exact decoder' : 'default decoder',
        );
      });
    }
  });

  group('min_bits', () {
    for (final r in records(RecordKind.minBits)) {
      test(r.type, () {
        expect(messageType(r.type).minBits, r.bits);
      });
    }
  });

  group('max_bits', () {
    for (final r in records(RecordKind.maxBits)) {
      test(r.type, () {
        expect(messageType(r.type).maxBits, r.maxBits);
      });
    }
    test('valid encodings are within max_bits', () {
      for (final r in records(RecordKind.valid)) {
        final m = messageType(r.type).maxBits;
        if (m != null) expect(r.bits, lessThanOrEqualTo(m), reason: r.name);
      }
    });
  });

  group('json roundtrip', () {
    for (final r in records(RecordKind.valid)) {
      test(r.name, () {
        expect(hex(messageType(r.type).jsonRoundtrip(r.data)), hex(r.data));
      });
    }
  });
}
