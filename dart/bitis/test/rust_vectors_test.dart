// Cross-checks against bytes produced by the Rust reference runtime.
import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

import 'rust_vectors.dart';
import 'type_spec.dart';

void main() {
  for (final v in rustVectors) {
    final types = [for (final (spec, _) in v.items) TS.parse(spec)];
    final values = v.items.map((e) => e.$2).toList();

    group(v.name, () {
      test('encode matches Rust bytes', () {
        final w = BitWriter();
        for (var i = 0; i < types.length; i++) {
          types[i].write(w, types[i].input(values[i]));
        }
        expect(bytesToHex(w.toBytes()), v.hex);
        expect(w.bitLength, v.bits);
      });

      test('decode Rust bytes', () {
        final bytes = hexToBytes(v.hex);
        final decoded = bitisDecodeExact(bytes, (r) {
          final out = <Object?>[];
          for (final t in types) {
            final before = r.bitPosition;
            out.add(t.read(r));
            expect(r.bitPosition - before, greaterThanOrEqualTo(t.minBits));
          }
          expect(r.bitPosition, v.bits);
          return out;
        });
        for (var i = 0; i < types.length; i++) {
          final exp = types[i].expected(values[i]);
          expect(
            types[i].same(decoded[i], exp),
            isTrue,
            reason: 'item $i (${v.items[i].$1}): got ${decoded[i]}, '
                'expected $exp',
          );
        }
      });

      test('every truncation fails with unexpectedEnd', () {
        final bytes = hexToBytes(v.hex);
        for (var len = 0; len < bytes.length; len++) {
          expect(
            () => bitisDecode(bytes.sublist(0, len), (r) {
              for (final t in types) {
                t.read(r);
              }
              return null;
            }),
            throwsA(
              isA<BitisDecodeException>().having(
                (e) => e.kind,
                'kind',
                BitisDecodeErrorKind.unexpectedEnd,
              ),
            ),
            reason: 'prefix length $len',
          );
        }
      });
    });
  }
}
