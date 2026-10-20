// Spot checks of decoded values and of the encoder for selected golden
// vectors (ported from tests/golden.rs and golden/cpp/golden_test.cpp).
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:test/test.dart';

import '../../gen/dart/golden.dart';
import 'manifest.dart';

final BigInt _u64Max = BigInt.parse('18446744073709551615');
final Int64 _i64Min = -Int64.MAX_VALUE; // the base type minimum (spec 2.3)

/// Bit pattern of a float (as f32) / double: [low 32 bits, high 32 bits]
/// (web-safe, no getUint64).
List<int> _f32Bits(double v) => [
  (ByteData(4)..setFloat32(0, v, Endian.little)).getUint32(0, Endian.little),
];
List<int> _f64Bits(double v) {
  final b = ByteData(8)..setFloat64(0, v, Endian.little);
  return [b.getUint32(0, Endian.little), b.getUint32(4, Endian.little)];
}

String _enc(BitisMessage m) => hex(bitisEncode(m));

final Matcher _outOfRange = throwsA(
  isA<BitisEncodeException>().having(
    (e) => e.kind,
    'kind',
    BitisEncodeErrorKind.valueOutOfRange,
  ),
);

void main() {
  group('decoded values', () {
    test('Ints', () {
      var m = decodeCase('ints_min', Ints.bitisRead);
      expect(
        [m.valI1, m.valI7, m.valI16, m.valI32],
        [-1, -127, -32767, -2147483647],
      );
      expect(m.valI33, Int64(-8589934591));
      expect(m.valI64, _i64Min);
      expect(m.valI64.toString(), '-9223372036854775807');

      m = decodeCase('ints_max', Ints.bitisRead);
      expect(m.valU33, Int64(8589934591));
      expect(m.valU63, Int64.MAX_VALUE);
      expect(m.valU64, _u64Max);
      expect(m.valI64, Int64.MAX_VALUE);
      expect(m.valU32, 4294967295);
      expect(m.valI32, 2147483647);

      m = decodeCase('noncanon_ints_negzero', Ints.bitisRead);
      expect([m.valI1, m.valI7, m.valI16], [0, 0, 0]);
      expect(m.valI64, Int64.ZERO);
    });

    test('DynInts', () {
      var m = decodeCase('dynints_min', DynInts.bitisRead);
      expect(m.valI16D5, -32767);
      expect(m.valI64D8, _i64Min);
      m = decodeCase('dynints_max', DynInts.bitisRead);
      expect(m.valU8D3, 255);
      expect(m.valU64D7, _u64Max);
      expect(m.valU64D64, _u64Max);
      m = decodeCase('noncanon_dynints_nonminimal', DynInts.bitisRead);
      expect([m.valU8D3, m.valU32D4, m.valI16D5], [5, 5, -3]);
    });

    test('Floats', () {
      var m = decodeCase('floats_negzero', Floats.bitisRead);
      expect(m.valF64 == 0.0 && m.valF64.isNegative, isTrue);
      expect(m.valF32 == 0.0 && m.valF32.isNegative, isTrue);
      m = decodeCase('floats_nan', Floats.bitisRead);
      expect(m.valF32.isNaN && m.valF64.isNaN, isTrue);
      m = decodeCase('floats_inf', Floats.bitisRead);
      expect(m.valF32, double.infinity);
      expect(m.valF64, double.negativeInfinity);
      m = decodeCase('floats_subnormal', Floats.bitisRead);
      expect(_f32Bits(m.valF32), [1]);
      expect(_f64Bits(m.valF64), [1, 0]);
    });

    test('FixedPoints', () {
      var m = decodeCase('fixedpoints_gnss', FixedPoints.bitisRead);
      expect(m.lat.value, closeTo(51.756349, 6e-9));
      expect(m.lng.value, closeTo(14.332868, 6e-9));
      // 0.49999999999999994 + 1.0 is 1.5 in f64 (spec 2.7), then rounded away
      // from zero
      expect(m.fp2.value, 1.0);
      expect(m.fp10.value, closeTo(-1.0, 0.0025));
      expect(m.fp53.value, closeTo(123456.789, 1e-9));
      m = decodeCase('fixedpoints_gnss_neg', FixedPoints.bitisRead);
      expect(m.lat.value, closeTo(-33.856784, 6e-9));
      expect(m.lng.value, closeTo(-151.215297, 6e-9));

      List<double?> values(FixedPoints m) => [
        m.fp2.valueOrNull,
        m.fp10.valueOrNull,
        m.lat.valueOrNull,
        m.lng.valueOrNull,
        m.fp53.valueOrNull,
      ];
      List<FixedPointState> states(FixedPoints m) => [
        m.fp2.state,
        m.fp10.state,
        m.lat.state,
        m.lng.state,
        m.fp53.state,
      ];
      m = decodeCase('fixedpoints_min', FixedPoints.bitisRead);
      expect(values(m), [0.0, -2.0, -90.0, -180.0, -1e6]);
      m = decodeCase('fixedpoints_max', FixedPoints.bitisRead);
      expect(values(m), [1.0, 3.0, 90.0, 180.0, 1e6]);
      m = decodeCase('fixedpoints_default', FixedPoints.bitisRead);
      expect(states(m), List.filled(5, FixedPointState.overflow));
      m = decodeCase('fixedpoints_overflow', FixedPoints.bitisRead);
      expect(states(m), List.filled(5, FixedPointState.overflow));
      m = decodeCase('fixedpoints_underflow', FixedPoints.bitisRead);
      expect(states(m), List.filled(5, FixedPointState.underflow));
      m = decodeCase('fixedpoints_zero', FixedPoints.bitisRead);
      expect(m.fp2.value, 1.0); // 0.5 is a rounding tie (t = 1.5)

      // the default value of a fixed point attribute is Overflow (as in Rust
      // and C++)
      expect(
        states(const FixedPoints()),
        List.filled(5, FixedPointState.overflow),
      );
    });

    test('Texts', () {
      var m = decodeCase('texts_values', Texts.bitisRead);
      expect(m.text, 'Hello, bitis!');
      expect(m.data, [0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff]);
      m = decodeCase('texts_ascii_edges', Texts.bitisRead);
      expect(m.text, '\x00 ~\x7f');
      m = decodeCase('texts_long', Texts.bitisRead);
      expect(m.text.length, 43);
      expect(m.data.length, 300);
      expect(m.data[299], 43);
      m = decodeCase('texts_empty', Texts.bitisRead);
      expect(m.text, isEmpty);
      expect(m.data, isEmpty);
    });

    test('Containers', () {
      var c = decodeCase('containers_full', Containers.bitisRead);
      expect(c.optColor, Color.yellow);
      expect(c.optText, '');
      expect(c.fixedInts, [-127, 0, 127]);
      expect(c.dynLeaves.length, 3);
      expect(c.dynLeaves[2].val, -63);
      expect(c.dynLeaves[0].tag, 'deep');
      c = decodeCase('containers_default', Containers.bitisRead);
      expect(c.optU7, isNull);
      expect(c.optLeaf, isNull);
      expect(c.optText, isNull);
      expect(c.optColor, isNull);
      expect(c.dynInts, isEmpty);
    });

    test('Enums', () {
      var e = decodeCase('enums_all', Enums.bitisRead);
      expect(e.color, Color.yellow);
      expect(e.level, Level.high);
      expect(e.colors.length, 6);
      expect(e.colors[5], Color.yellow);
      e = decodeCase('enums_default', Enums.bitisRead);
      expect(e.color, Color.green);
      expect(e.level, Level.low);
    });

    test('Choice', () {
      var ch = decodeCase('choice_text', Choice.bitisRead);
      expect(
        ch.value,
        isA<OO_Choice_Value_Text>().having(
          (v) => v.value,
          'value',
          'oneof text',
        ),
      );
      expect(ch.tail, isTrue);
      ch = decodeCase('choice_big', Choice.bitisRead);
      expect(
        ch.value,
        isA<OO_Choice_Value_Big>().having((v) => v.value, 'value', -2147483647),
      );
      ch = decodeCase('choice_list', Choice.bitisRead);
      expect(
        ch.value,
        isA<OO_Choice_Value_List>().having((v) => v.value.items, 'items', [
          -1,
          0,
          1,
          127,
          -127,
        ]),
      );
      ch = decodeCase('choice_nothing', Choice.bitisRead);
      expect(ch.value, isA<OO_Choice_Value_Nothing>());
      expect(ch.tail, isTrue);
      ch = decodeCase('choice_default', Choice.bitisRead);
      expect(ch.value, const OO_Choice_Value_Flag(false));
      expect(ch.tail, isFalse);
    });

    test('Nested / Everything', () {
      final n = decodeCase('nested_deep', Nested.bitisRead);
      expect(n.mid.id, 99);
      expect(n.mid.leaf.val, -42);
      expect(n.mid.leaf.tag, 'level3');
      expect(n.mid.extra?.tag, 'extra');
      expect(n.after, isTrue);

      final all = decodeCase('everything_full', Everything.bitisRead);
      expect(all.tail, 5);
      expect(all.ints.valI64, Int64.MAX_VALUE);
      expect(all.nested.mid.leaf.val, -42);
    });
  });

  group('encoder', () {
    test('fixed point quantization incl. exact rounding ties', () {
      // round half away from zero
      const ties = FixedPoints(
        fp2: FixedPoint.value(0.5),
        fp10: FixedPoint.value(-1.845739471106758),
        lat: FixedPoint.value(-1.5684636309743063e-05),
        lng: FixedPoint.value(-1.5684636309743063e-05),
        fp53: FixedPoint.value(-3.32365743815901e-07),
      );
      expect(_enc(ties), hex(caseBytes('fixedpoints_ties')));
      const gnss = FixedPoints(
        fp2: FixedPoint.value(0.49999999999999994),
        fp10: FixedPoint.value(-1.0),
        lat: FixedPoint.value(51.756349),
        lng: FixedPoint.value(14.332868),
        fp53: FixedPoint.value(123456.789),
      );
      expect(_enc(gnss), hex(caseBytes('fixedpoints_gnss')));
      const under = FixedPoints(
        fp2: FixedPoint.value(double.nan),
        fp10: FixedPoint.value(-2.001),
        lat: FixedPoint.value(-90.000001),
        lng: FixedPoint.value(double.negativeInfinity),
        fp53: FixedPoint.value(-1000000.001),
      );
      expect(_enc(under), hex(caseBytes('fixedpoints_underflow')));
      const over = FixedPoints(
        fp2: FixedPoint.value(1.0000001),
        fp10: FixedPoint.overflow(),
        lat: FixedPoint.value(90.000001),
        lng: FixedPoint.value(double.infinity),
        fp53: FixedPoint.value(1000000.001),
      );
      expect(_enc(over), hex(caseBytes('fixedpoints_overflow')));
      expect(_enc(const FixedPoints()), hex(caseBytes('fixedpoints_default')));
    });

    test('integers, texts and defaults', () {
      final ints = Ints(
        valI1: -1,
        valI7: -127,
        valI16: -32767,
        valI32: -2147483647,
        valI33: Int64(-8589934591),
        valI64: _i64Min,
      );
      expect(_enc(ints), hex(caseBytes('ints_min')));
      final texts = Texts(
        text: 'Hello, bitis!',
        data: Uint8List.fromList([0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff]),
      );
      expect(_enc(texts), hex(caseBytes('texts_values')));
      expect(const Empty().toBytes(), isEmpty);
      expect(_enc(Everything()), hex(caseBytes('everything_default')));
      expect(_enc(const Choice()), hex(caseBytes('choice_default')));
      expect(_enc(Ints()), hex(caseBytes('ints_default')));
    });

    test('values outside the valid range throw valueOutOfRange', () {
      expect(() => Ints(valI64: Int64.MIN_VALUE).toBytes(), _outOfRange);
      expect(() => Ints(valI16: -32768).toBytes(), _outOfRange);
      expect(() => DynInts(valI16D5: -32768).toBytes(), _outOfRange);
      expect(() => Ints(valU7: 128).toBytes(), _outOfRange);
      expect(() => Texts(text: 'ä').toBytes(), _outOfRange);
    });
  });
}
