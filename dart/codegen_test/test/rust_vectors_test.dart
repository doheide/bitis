// Cross-check against the Rust reference runtime: the bytes of the values in
// fixtures/vector_inputs.json (JSON mapping) and of the default value of every
// message, produced by tool/rust_vectors (see tool/regen.sh --vectors).
import 'dart:convert';

import 'package:bitis/bitis.dart';
import 'package:bitis_codegen_test/all_types.dart' as all_types;
import 'package:bitis_codegen_test/cara.dart' as cara;
import 'package:bitis_codegen_test/test_schema.dart' as test_schema;
import 'package:test/test.dart';

import 'messages.g.dart';
import 'rust_vectors.g.dart';

String hex(List<int> b) =>
    b.map((x) => x.toRadixString(16).padLeft(2, '0')).join();

List<int> unhex(String s) => [
  for (var i = 0; i < s.length; i += 2)
    int.parse(s.substring(i, i + 2), radix: 16),
];

void main() {
  final fixture = BitisJson.objectFromJson(jsonDecode(rustVectorsJson));
  final vectors = BitisJson.listFromJson(fixture['vectors']);

  test('schema hashes match the Rust code', () {
    final hashes = BitisJson.objectFromJson(fixture['schemas']);
    expect(hashes['cara'], cara.bitisSchemaHashHex);
    expect(hashes['test_schema'], test_schema.bitisSchemaHashHex);
    expect(hashes['all_types'], all_types.bitisSchemaHashHex);
  });

  test('every message has a default vector', () {
    for (final schema in ['cara', 'test_schema', 'all_types']) {
      for (final m in messages[schema]!.keys) {
        expect(
          vectors.any(
            (v) => (v as Map)['name'] == 'default_$m' && v['schema'] == schema,
          ),
          isTrue,
          reason: '$schema $m',
        );
      }
    }
  });

  test('every message has a non-default vector', () {
    for (final schema in ['cara', 'test_schema', 'all_types']) {
      for (final m in messages[schema]!.keys) {
        expect(
          vectors.any(
            (v) =>
                (v as Map)['message'] == m &&
                v['schema'] == schema &&
                v['value'] != null,
          ),
          isTrue,
          reason: '$schema $m',
        );
      }
    }
  });

  for (final v in vectors.map(BitisJson.objectFromJson)) {
    final name = '${v['schema']} ${v['name']}';
    test(name, () {
      final ops = messages[v['schema']]![v['message']]!;
      final value = v['value'];
      final msg =
          value == null
              ? ops.create()
              : ops.fromJson(BitisJson.objectFromJson(value));
      final expectedHex = v['hex'] as String;
      final expectedBits = v['bits'] as int;

      // encode: same bytes as Rust
      final enc = bitisEncodeWithSize(msg);
      expect(hex(enc.bytes), expectedHex, reason: 'encoding of $msg');
      expect(enc.totalBits, expectedBits);
      expect(enc.totalBits, greaterThanOrEqualTo(ops.minBits));

      // decode the Rust bytes
      final bytes = unhex(expectedHex);
      final decoded = ops.fromBytesExact(bytes);
      expect(hex(bitisEncode(decoded)), expectedHex);
      final hasNaN = jsonEncode(value).contains('NaN');
      if (!hasNaN) {
        expect(decoded, equals(msg));
        expect(decoded.hashCode, msg.hashCode);
      }

      // JSON of the decoded value (through a JSON text) encodes to the same bytes
      final json = jsonDecode(jsonEncode(ops.toJson(decoded)));
      final fromJson = ops.fromJson(BitisJson.objectFromJson(json));
      expect(hex(bitisEncode(fromJson)), expectedHex);
      if (!hasNaN) expect(fromJson, equals(decoded));
    });
  }
}
