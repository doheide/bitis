import 'int_util.dart';

/// State of a [FixedPoint] value.
enum FixedPointState {
  /// Below the declared minimum (raw value 0), or NaN when encoded.
  underflow,

  /// A value inside `[min, max]`.
  value,

  /// Above the declared maximum (raw value `2^N - 1`).
  overflow,
}

/// Value of a fixed precision field `fp_N[min, max]` (spec 2.7).
///
/// Underflow and overflow are explicit states, not numeric sentinels.
/// A [FixedPoint] does not know its field type; use
/// [FixedPointType.fromDouble] to get the value as it will be decoded on the
/// other side (classified and quantized).
final class FixedPoint {
  /// A value in physical units. Values outside the field's `[min, max]` are
  /// encoded as overflow/underflow, NaN is encoded as underflow.
  const FixedPoint.value(double value)
      : state = FixedPointState.value,
        _value = value;

  /// The underflow state (raw value 0).
  const FixedPoint.underflow()
      : state = FixedPointState.underflow,
        _value = null;

  /// The overflow state (raw value `2^N - 1`).
  const FixedPoint.overflow()
      : state = FixedPointState.overflow,
        _value = null;

  /// The state of this value.
  final FixedPointState state;
  final double? _value;

  /// True if [state] is [FixedPointState.value].
  bool get isValue => state == FixedPointState.value;

  /// True if [state] is [FixedPointState.underflow].
  bool get isUnderflow => state == FixedPointState.underflow;

  /// True if [state] is [FixedPointState.overflow].
  bool get isOverflow => state == FixedPointState.overflow;

  /// The value in physical units, or null for underflow/overflow.
  double? get valueOrNull => _value;

  /// The value in physical units. Throws [StateError] for underflow/overflow.
  double get value {
    final v = _value;
    if (v == null) {
      throw StateError('FixedPoint is ${state.name}, not a value');
    }
    return v;
  }

  @override
  bool operator ==(Object other) =>
      other is FixedPoint && other.state == state && other._value == _value;

  @override
  int get hashCode => Object.hash(state, _value);

  @override
  String toString() => switch (state) {
        FixedPointState.value => 'FixedPoint($_value)',
        _ => 'FixedPoint.${state.name}',
      };
}

/// The field type `fp_N[min, max]`: [bits] = N (2..53), integer [min] < [max].
///
/// Generated code declares one constant per field, e.g.
/// `static const _fpTemp = FixedPointType(10, -40, 125);`.
///
/// [min] and [max] must be within ±2^53 (exact as double), which is required
/// for the Dart web target anyway.
final class FixedPointType {
  /// Creates the type `fp_bits[min, max]`.
  const FixedPointType(this.bits, this.min, this.max)
      : assert(bits >= 2 && bits <= 53, 'fp_N needs 2 <= N <= 53'),
        assert(min < max, 'fp_N[min, max] needs min < max');

  /// Number of bits N.
  final int bits;

  /// Lower bound (inclusive) in physical units.
  final int min;

  /// Upper bound (inclusive) in physical units.
  final int max;

  /// Raw value used for overflow: `2^N - 1`.
  int get maxRaw => pow2[bits] - 1;

  /// `S = (2^N - 3) as f64`.
  double get _scale => (pow2[bits] - 3).toDouble();

  /// `R = max - min` computed as in the Rust reference
  /// (`max as f64 - min as f64`).
  double get _range => max.toDouble() - min.toDouble();

  /// Quantization step in physical units (`R / S`).
  double get resolution => _range / _scale;

  /// Raw N-bit value for [v] using exactly the f64 formula of spec 2.7.
  int encodeRaw(FixedPoint v) {
    switch (v.state) {
      case FixedPointState.underflow:
        return 0;
      case FixedPointState.overflow:
        return maxRaw;
      case FixedPointState.value:
        final x = v._value!;
        final minD = min.toDouble();
        if (x > max.toDouble()) return maxRaw;
        if (x < minD) return 0;
        if (x.isNaN) return 0;
        final t = (x - minD) / _range * _scale + 1.0;
        return roundHalfAwayFromZero(t);
    }
  }

  /// Value for the raw N-bit value [raw] (`0 <= raw <= 2^N - 1`).
  FixedPoint decodeRaw(int raw) {
    if (raw == 0) return const FixedPoint.underflow();
    if (raw == maxRaw) return const FixedPoint.overflow();
    return FixedPoint.value(
      (raw - 1).toDouble() / _scale * _range + min.toDouble(),
    );
  }

  /// [v] as the receiver will see it after encoding and decoding.
  FixedPoint quantize(FixedPoint v) => decodeRaw(encodeRaw(v));

  /// Classifies and quantizes the physical value [v]: overflow above [max],
  /// underflow below [min] or for NaN, otherwise the nearest representable
  /// value.
  FixedPoint fromDouble(double v) => quantize(FixedPoint.value(v));

  /// True if [a] and [b] encode to the same raw value (the equality used by
  /// the Rust runtime).
  bool sameEncoding(FixedPoint a, FixedPoint b) => encodeRaw(a) == encodeRaw(b);

  @override
  bool operator ==(Object other) =>
      other is FixedPointType &&
      other.bits == bits &&
      other.min == min &&
      other.max == max;

  @override
  int get hashCode => Object.hash(bits, min, max);

  @override
  String toString() => 'fp_$bits[$min, $max]';
}

/// Rounds a finite, non-negative double to the nearest integer, ties away
/// from zero (Rust `f64::round`), without relying on platform rounding.
///
/// For `t >= 1`, `t - floor(t)` is exact (Sterbenz), so the tie detection is
/// exact. Only used with `1 <= t <= 2^53 - 2`.
int roundHalfAwayFromZero(double t) {
  final f = t.floorToDouble();
  return ((t - f) >= 0.5 ? f + 1.0 : f).toInt();
}
