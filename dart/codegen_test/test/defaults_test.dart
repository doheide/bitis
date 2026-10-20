// Generic tests over every generated message (all schemas of fixtures/schemas):
// default values, byte and JSON round trips, equality, error handling.
import 'dart:convert';
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

import 'messages.g.dart';

Matcher throwsDecode(BitisDecodeErrorKind kind) =>
    throwsA(isA<BitisDecodeException>().having((e) => e.kind, 'kind', kind));

void main() {
  for (final MapEntry(key: schema, value: msgs) in messages.entries) {
    group(schema, () {
      for (final MapEntry(key: name, value: ops) in msgs.entries) {
        test('$name default value', () {
          final msg = ops.create();
          final enc = bitisEncodeWithSize(msg);
          expect(enc.totalBits, greaterThanOrEqualTo(ops.minBits));
          if (ops.maxBits case final max?) {
            expect(enc.totalBits, lessThanOrEqualTo(max));
          }
          expect(enc.bytes.length, (enc.totalBits + 7) ~/ 8);

          // byte round trip, equality and hash code
          final decoded = ops.fromBytesExact(enc.bytes);
          expect(decoded, equals(msg));
          expect(decoded.hashCode, msg.hashCode);
          expect(ops.create(), equals(msg));
          expect(decoded.toString(), msg.toString());
          expect(decoded.toString(), startsWith('$name('));

          // trailing bytes: ignored by fromBytes, rejected by fromBytesExact
          final longer = Uint8List.fromList([...enc.bytes, 0xff, 0x00]);
          expect(ops.fromBytes(longer), equals(msg));
          expect(
            () => ops.fromBytesExact(longer),
            throwsDecode(BitisDecodeErrorKind.trailingData),
          );

          // truncated input: every byte contains significant bits
          if (enc.bytes.isNotEmpty) {
            final short = enc.bytes.sublist(0, enc.bytes.length - 1);
            expect(
              () => ops.fromBytes(short),
              throwsDecode(BitisDecodeErrorKind.unexpectedEnd),
            );
          }

          // JSON round trip through a JSON text
          final json = jsonDecode(jsonEncode(ops.toJson(msg)));
          final fromJson = ops.fromJson(BitisJson.objectFromJson(json));
          expect(fromJson, equals(msg));
          expect(bitisEncode(fromJson), enc.bytes);
        });

        test('$name decodes random input or fails cleanly', () {
          // decoding untrusted input only throws BitisDecodeException
          var state = name.hashCode & 0x7fffffff;
          int next() => state = (state * 1103515245 + 12345) & 0x7fffffff;
          for (var i = 0; i < 50; i++) {
            final data = List<int>.generate(next() % 24, (_) => next() & 0xff);
            try {
              final m = ops.fromBytes(data);
              // a valid decoding re-encodes to a prefix-compatible value
              expect(ops.fromBytes(bitisEncode(m)), equals(m));
            } on BitisDecodeException catch (_) {
              // expected for most inputs
            }
          }
        });
      }
    });
  }
}
