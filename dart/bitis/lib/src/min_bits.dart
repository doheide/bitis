/// `min_bits(T)` per spec section 4: a lower bound of the encoded size of a
/// type (exact for fixed-size types), identical in all runtimes.
///
/// Used as the `elementMinBits` argument of [BitReader.readDynArray].
/// Generated code typically emits the computed number as
/// `static const int bitisMinBits = ...;` per message.
abstract final class MinBits {
  /// `bool`.
  static const int boolean = 1;

  /// `uint_X`.
  static int uint(int bits) => bits;

  /// `int_X` (sign bit + magnitude).
  static int sint(int bits) => bits + 1;

  /// `uint_XdY`.
  static const int dynUint = 1;

  /// `int_XdY`.
  static const int dynInt = 2;

  /// `float`.
  static const int float32 = 32;

  /// `double`.
  static const int float64 = 64;

  /// `fp_N[min, max]`.
  static int fixedPoint(int bits) => bits;

  /// `astr_dY` (count zero).
  static const int astr = 1;

  /// `binary_dY` (count zero).
  static const int binary = 1;

  /// `repeated_dyn_Y T` (count zero).
  static const int dynArray = 1;

  /// `optional T`.
  static const int optional = 1;

  /// `enum E(Y)`.
  static const int enumeration = 1;

  /// `repeated_fixed_N T`: `N * min_bits(T)`.
  static int fixedArray(int length, int elementMinBits) =>
      length * elementMinBits;

  /// `oneof`: `1 + min over variants of min_bits(variant)`.
  static int oneOf(Iterable<int> variantMinBits) {
    var m = -1;
    for (final v in variantMinBits) {
      if (m < 0 || v < m) m = v;
    }
    if (m < 0) throw ArgumentError('oneof needs at least one variant');
    return 1 + m;
  }

  /// `msg`: sum of the attributes' min_bits.
  static int message(Iterable<int> attributeMinBits) =>
      attributeMinBits.fold(0, (a, b) => a + b);
}
