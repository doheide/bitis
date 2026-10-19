// Error kinds, truncation and a deterministic random-buffer fuzz loop.
import 'dart:math';
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

import 'bits.dart';
import 'type_spec.dart';

final _end = throwsDecode(BitisDecodeErrorKind.unexpectedEnd);
final _invalid = throwsDecode(BitisDecodeErrorKind.invalidValue);
final _disc = throwsDecode(BitisDecodeErrorKind.invalidDiscriminant);

class _Msg extends BitisMessage {
  _Msg(this.a, this.b);
  final int a; // uint_5
  final String b; // astr_d4

  static _Msg bitisRead(BitReader r) => _Msg(r.readUint(5), r.readAStr(4));

  @override
  void bitisWrite(BitWriter w) {
    w.writeUint(a, 5);
    w.writeAStr(b, 4);
  }
}

void main() {
  group('unexpectedEnd', () {
    test('every read past the end', () {
      BitReader e() => BitReader(Uint8List(0));
      expect(() => e().readBool(), _end);
      expect(() => e().readUint(1), _end);
      expect(() => e().readInt(1), _end);
      expect(() => e().readUintInt64(40), _end);
      expect(() => e().readUintBigInt(64), _end);
      expect(() => e().readIntInt64(64), _end);
      expect(() => e().readDynUint(32, 3), _end);
      expect(() => e().readDynInt(32, 3), _end);
      expect(() => e().readFloat32(), _end);
      expect(() => e().readFloat64(), _end);
      expect(() => e().readFixedPoint(const FixedPointType(4, 0, 1)), _end);
      expect(() => e().readAStr(4), _end);
      expect(() => e().readBinary(4), _end);
      expect(() => e().readOptional((r) => r.readBool()), _end);
      expect(() => e().readDynArray(4, 1, (r) => r.readBool()), _end);
      expect(() => e().readEnumIndex(2, 2), _end);
      expect(() => e().readOneOfIndex(2, 2), _end);
      expect(() => e().readFixedArray(1, (r) => r.readBool()), _end);
      // no implicit zero bits: 7 bits are not enough for a uint_8
      final r = BitReader([0xFF])..readBool();
      expect(() => r.readUint(8), _end);
      expect(r.remainingBits, 7);
    });

    test('partial values', () {
      // flag set but chunk missing
      final r = BitReader([0xFF, 0xFF]);
      r.readUint(15);
      expect(() => r.readDynUint(32, 3), _end);
      expect(() => BitReader([0, 0, 0]).readFloat32(), _end);
      expect(() => BitReader(Uint8List(7)).readFloat64(), _end);
      expect(() => BitReader(Uint8List(8)).readUintBigInt(64), returnsNormally);
      expect(() => BitReader(Uint8List(8)).readIntInt64(64), _end);
    });

    test('count cap before allocation (spec section 3)', () {
      // astr with count 2^32-1 and nothing behind it
      final w = BitWriter()..writeDynUint(0xFFFFFFFF, 32, 8);
      expect(() => BitReader(w.toBytes()).readAStr(8), _end);
      expect(() => BitReader(w.toBytes()).readBinary(8), _end);
      expect(
        () => BitReader(w.toBytes()).readDynArray(8, 1, (r) => r.readBool()),
        _end,
      );
      // count 3 of 8-bit bytes with 23 bits left
      final w2 = BitWriter()
        ..writeDynUint(3, 32, 6) // 8 bits
        ..writeUint(0xFFFF, 16);
      expect(w2.bitLength, 24);
      expect(() => BitReader(w2.toBytes()).readBinary(6), _end);
      expect(
          () => BitReader([...w2.toBytes(), 0]).readBinary(6), returnsNormally);
      // dyn array with element min_bits 64: 2 elements need 128 bits
      final w3 = BitWriter()..writeDynUint(2, 32, 8);
      final bytes = Uint8List(w3.byteLength + 15)..setAll(0, w3.toBytes());
      var called = 0;
      expect(
        () => BitReader(bytes).readDynArray(8, 64, (r) {
          called++;
          return r.readFloat64();
        }),
        _end,
      );
      expect(called, 0);
      // element min_bits 0 is treated as 1
      final w4 = BitWriter()..writeDynUint(1000, 32, 16);
      expect(() => BitReader(w4.toBytes()).readDynArray(16, 0, (r) => 0), _end);
    });
  });

  group('invalidValue', () {
    test('int_X magnitude above the base type range', () {
      // int_16: magnitude 32768 (i16::MIN) is invalid, also with sign 1
      expect(() => BitReader(bitsToBytes('0 ${lsb(32768, 16)}')).readInt(16),
          _invalid);
      expect(() => BitReader(bitsToBytes('1 ${lsb(32768, 16)}')).readInt(16),
          _invalid);
      expect(() => BitReader(bitsToBytes('1 ${lsb(65535, 16)}')).readInt(16),
          _invalid);
      expect(
          () => BitReader(bitsToBytes('0 ${lsb(0x80000000, 32)}')).readInt(32),
          _invalid);
      expect(
        () =>
            BitReader(bitsToBytes('1 ${lsb(0x80000000, 32)}')).readIntInt64(32),
        _invalid,
      );
      final m63 = BigInt.one << 63;
      expect(() => BitReader(bitsToBytes('1 ${lsb(m63, 64)}')).readIntInt64(64),
          _invalid);
      expect(
        () => BitReader(
                bitsToBytes('0 ${lsb((BigInt.one << 64) - BigInt.one, 64)}'))
            .readIntInt64(64),
        _invalid,
      );
      // int_15 uses all magnitude bits: 32767 is valid
      expect(BitReader(bitsToBytes('1 ${lsb(32767, 15)}')).readInt(15), -32767);
    });

    test('int_XdY magnitude above the base type range', () {
      final w = BitWriter()
        ..writeBool(false)
        ..writeDynUint(32768, 16, 4);
      expect(() => BitReader(w.toBytes()).readDynInt(16, 4), _invalid);
      final w2 = BitWriter()
        ..writeBool(true)
        ..writeDynUintBigInt(BigInt.one << 63, 64, 8);
      expect(() => BitReader(w2.toBytes()).readDynIntInt64(64, 8), _invalid);
    });

    test('astr byte >= 0x80', () {
      final w = BitWriter()..writeBinary([0x41, 0x80], 4);
      expect(() => BitReader(w.toBytes()).readAStr(4), _invalid);
      final w2 = BitWriter()
        ..writeBool(true)
        ..writeBinary([0xFF], 4);
      final r = BitReader(w2.toBytes())..readBool();
      expect(() => r.readAStr(4), _invalid);
    });
  });

  group('invalidDiscriminant', () {
    test('enum / oneof index >= count', () {
      final w = BitWriter()..writeDynUint(3, 32, 2);
      expect(() => BitReader(w.toBytes()).readEnumIndex(3, 2), _disc);
      expect(() => BitReader(w.toBytes()).readOneOfIndex(3, 2), _disc);
      expect(BitReader(w.toBytes()).readEnumIndex(4, 2), 3);
      final big = BitWriter()..writeDynUint(0xFFFFFFFF, 32, 32);
      expect(() => BitReader(big.toBytes()).readEnumIndex(1, 32), _disc);
    });
  });

  group('top-level encode / decode', () {
    test('roundtrip with sizes', () {
      final m = _Msg(17, 'ok');
      final enc = bitisEncodeWithSize(m);
      expect(enc.totalBits, 5 + 6 + 16);
      expect(enc.totalBytes, 4);
      expect(bitisEncode(m), enc.bytes);
      final dec = bitisDecode(enc.bytes, _Msg.bitisRead);
      expect(dec.value.a, 17);
      expect(dec.value.b, 'ok');
      expect(dec.totalBits, 27);
      expect(dec.totalBytes, 4);
    });

    test('trailing data', () {
      final bytes = [...bitisEncode(_Msg(1, 'x')), 0x00];
      final dec = bitisDecode(bytes, _Msg.bitisRead);
      expect(dec.totalBytes, bytes.length - 1);
      expect(
        () => bitisDecodeExact(bytes, _Msg.bitisRead),
        throwsDecode(BitisDecodeErrorKind.trailingData),
      );
      expect(
          bitisDecodeExact(bytes.sublist(0, bytes.length - 1), _Msg.bitisRead)
              .b,
          'x');
    });

    test('padding bits are not checked', () {
      final bytes = bitisEncode(_Msg(1, 'x'));
      bytes[bytes.length - 1] |= 0x80;
      expect(bitisDecodeExact(bytes, _Msg.bitisRead).b, 'x');
    });

    test('empty message encodes to 0 bytes', () {
      expect(bitisDecodeExact(<int>[], (r) => 42), 42);
      expect(
        () => bitisDecodeExact([0], (r) => 42),
        throwsDecode(BitisDecodeErrorKind.trailingData),
      );
    });

    test('truncation at every length', () {
      final bytes = bitisEncode(_Msg(31, 'hello world'));
      for (var n = 0; n < bytes.length; n++) {
        expect(() => bitisDecode(bytes.sublist(0, n), _Msg.bitisRead), _end);
      }
    });
  });

  group('encode errors (valueOutOfRange)', () {
    test('integers', () {
      expect(() => BitWriter().writeUint(32, 5), throwsEncode);
      expect(() => BitWriter().writeUint(-1, 5), throwsEncode);
      expect(() => BitWriter().writeUint(4294967296, 32), throwsEncode);
      expect(() => BitWriter().writeInt(-32768, 16), throwsEncode);
      expect(() => BitWriter().writeInt(32768, 16), throwsEncode);
      expect(() => BitWriter().writeInt(-32, 5), throwsEncode);
      expect(() => BitWriter().writeInt(-2147483648, 32), throwsEncode);
      expect(() => BitWriter().writeInt(65535, 16), throwsEncode);
      expect(
          () => BitWriter().writeIntInt64(Int64.MIN_VALUE, 64), throwsEncode);
      expect(() => BitWriter().writeIntInt64(Int64(32768), 16), throwsEncode);
      expect(() => BitWriter().writeIntInt64(Int64(1) << 40, 40), throwsEncode);
      expect(() => BitWriter().writeUintInt64(Int64(-1), 40), throwsEncode);
      expect(
          () => BitWriter().writeUintInt64(Int64(1) << 40, 40), throwsEncode);
      expect(() => BitWriter().writeUintBigInt(BigInt.one << 64, 64),
          throwsEncode);
      expect(() => BitWriter().writeUintBigInt(-BigInt.one, 64), throwsEncode);
      expect(() => BitWriter().writeDynUint(256, 8, 3), throwsEncode);
      expect(() => BitWriter().writeDynInt(-32768, 16, 3), throwsEncode);
      expect(() => BitWriter().writeDynIntInt64(Int64.MIN_VALUE, 64, 8),
          throwsEncode);
      expect(
          () => BitWriter().writeDynUintInt64(Int64(-5), 63, 8), throwsEncode);
      expect(() => BitWriter().writeDynUintBigInt(BigInt.one << 64, 64, 8),
          throwsEncode);
    });

    test('strings, binary, arrays, indices', () {
      expect(() => BitWriter().writeAStr('Grüße', 4), throwsEncode);
      expect(() => BitWriter().writeAStr('\u0080', 4), throwsEncode);
      expect(() => BitWriter().writeBinary([256], 4), throwsEncode);
      expect(() => BitWriter().writeBinary([-1], 4), throwsEncode);
      expect(
        () =>
            BitWriter().writeFixedArray([1, 2], 3, (w, v) => w.writeUint(v, 2)),
        throwsEncode,
      );
      expect(() => BitWriter().writeEnumIndex(3, 3, 2), throwsEncode);
      expect(() => BitWriter().writeOneOfIndex(-1, 3, 2), throwsEncode);
    });

    test('invalid type parameters are ArgumentErrors', () {
      expect(() => BitWriter().writeUint(0, 33), throwsArgumentError);
      expect(() => BitWriter().writeUint(0, 0), throwsArgumentError);
      expect(() => BitWriter().writeUintInt64(Int64.ZERO, 64),
          throwsArgumentError);
      expect(() => BitWriter().writeDynUint(0, 8, 9), throwsArgumentError);
      expect(() => BitReader([0]).readUint(33), throwsArgumentError);
      expect(() => BitReader([0]).readDynUint(8, 0), throwsArgumentError);
    });
  });

  group('fuzz', () {
    // A schema touching every type; random buffers must decode or fail with
    // a BitisDecodeException, never anything else.
    final schemas = [
      'bool',
      'u5',
      'u32',
      'u40',
      'u64',
      'i16',
      'i32',
      'i64',
      'du32d3',
      'du64d8',
      'du64d64',
      'di16d4',
      'di64d7',
      'f32',
      'f64',
      'fp10[-5,100]',
      'fp40[-1000000,1000000]',
      'fp53[-1,1]',
      'astr4',
      'bin3',
      'opt<u7>',
      'fa3<i5>',
      'da2<bool>',
      'da3<da2<du32d3>>',
      'da8<astr2>',
      'opt<da4<f32>>',
    ].map(TS.parse).toList();

    test('random buffers only throw BitisDecodeException', () {
      final rnd = Random(20261019);
      var ok = 0;
      final kinds = <BitisDecodeErrorKind>{};
      for (var iter = 0; iter < 3000; iter++) {
        final len = rnd.nextInt(40);
        final bytes =
            Uint8List.fromList(List.generate(len, (_) => rnd.nextInt(256)));
        // a random suffix of the schema list, so every type is hit first
        final start = rnd.nextInt(schemas.length);
        final seq = [...schemas.sublist(start), ...schemas.sublist(0, start)];
        try {
          final values =
              bitisDecode(bytes, (r) => [for (final t in seq) t.read(r)]).value;
          ok++;
          // re-encode the decoded values and decode them again
          final w = BitWriter();
          for (var i = 0; i < seq.length; i++) {
            seq[i].write(w, values[i]);
          }
          final again = bitisDecodeExact(
              w.toBytes(), (r) => [for (final t in seq) t.read(r)]);
          for (var i = 0; i < seq.length; i++) {
            expect(seq[i].same(again[i], values[i]), isTrue);
          }
        } on BitisDecodeException catch (e) {
          kinds.add(e.kind);
        }
      }
      expect(kinds, contains(BitisDecodeErrorKind.unexpectedEnd));
      expect(kinds, contains(BitisDecodeErrorKind.invalidValue));
      expect(ok, greaterThanOrEqualTo(0));
    });

    test('per-type random buffers', () {
      final rnd = Random(42);
      for (final t in schemas) {
        for (var iter = 0; iter < 300; iter++) {
          final bytes = Uint8List.fromList(
            List.generate(rnd.nextInt(24), (_) => rnd.nextInt(256)),
          );
          try {
            final r = BitReader(bytes);
            t.read(r);
            expect(r.bitPosition, lessThanOrEqualTo(r.totalBits));
          } on BitisDecodeException {
            // expected
          }
        }
      }
    });

    test('random valid values roundtrip', () {
      final rnd = Random(7);
      for (var iter = 0; iter < 300; iter++) {
        final values = [for (final t in schemas) t.random(rnd)];
        final w = BitWriter();
        for (var i = 0; i < schemas.length; i++) {
          schemas[i].write(w, values[i]);
        }
        final bytes = w.toBytes();
        final back = bitisDecodeExact(bytes, (r) {
          final out = <Object?>[];
          for (final t in schemas) {
            final p = r.bitPosition;
            out.add(t.read(r));
            expect(r.bitPosition - p, greaterThanOrEqualTo(t.minBits));
          }
          expect(r.bitPosition, w.bitLength);
          return out;
        });
        for (var i = 0; i < schemas.length; i++) {
          // floats: NaN payloads are canonicalized on encode
          expect(schemas[i].same(back[i], values[i]), isTrue,
              reason: '$i ${values[i]}');
        }
        // every truncation fails with unexpectedEnd
        for (var n = 0; n < bytes.length; n += 1 + n ~/ 4) {
          expect(
            () => bitisDecode(bytes.sublist(0, n),
                (r) => [for (final t in schemas) t.read(r)]),
            _end,
          );
        }
      }
    });
  });
}
