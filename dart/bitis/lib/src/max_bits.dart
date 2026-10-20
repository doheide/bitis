/// `max_bits(T)` per spec section 4: an upper bound of the size of the
/// encoding an encoder produces for any valid value of a type, identical in
/// all runtimes. `null` means unbounded.
///
/// Results are exact integers. A result above [maxSafe] (`2^53 - 1`, the
/// largest integer that is exact with dart2js) is returned as `null`, i.e.
/// treated as unbounded; the Rust runtime likewise returns `None` on
/// overflow. Generated code emits the computed number as
/// `static const int? bitisMaxBits = ...;` per message.
abstract final class MaxBits {
  /// Largest result that is returned as a number (`2^53 - 1`).
  static const int maxSafe = 9007199254740991;

  /// Size in bits of the `uint_XdY` encoding of [value] (spec 2.4) with
  /// `X = maxBits`, `Y = chunkBits`. [value] must be in `0 .. 2^53 - 1` and
  /// fit into X bits.
  static int dyn(int value, int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits);
    if (value < 0 || value > maxSafe) {
      throw ArgumentError.value(value, 'value', 'must be in 0..2^53-1');
    }
    if (value == 0) return 1;
    final len = value.bitLength;
    if (len > maxBits) {
      throw ArgumentError.value(value, 'value', 'does not fit $maxBits bits');
    }
    final chunks = (len + chunkBits - 1) ~/ chunkBits;
    final dataBits =
        chunks * chunkBits < maxBits ? chunks * chunkBits : maxBits;
    // a continuation bit follows every chunk unless the X bits are used up
    final continuation = dataBits < maxBits ? chunks : chunks - 1;
    return 1 + dataBits + continuation;
  }

  static void _checkDyn(int maxBits, int chunkBits) {
    if (maxBits < 1 || maxBits > 64) {
      throw ArgumentError.value(maxBits, 'maxBits', 'must be in 1..64');
    }
    if (chunkBits < 1 || chunkBits > maxBits) {
      throw ArgumentError.value(
        chunkBits,
        'chunkBits',
        'must be in 1..$maxBits',
      );
    }
  }

  /// `dynmax(X, Y) = dyn(2^X - 1, X, Y) = X + ceil(X / Y)`.
  static int _dynMax(int maxBits, int chunkBits) {
    _checkDyn(maxBits, chunkBits);
    return maxBits + (maxBits + chunkBits - 1) ~/ chunkBits;
  }

  /// `a + b`; null if one of them is null or the sum exceeds [maxSafe].
  static int? add(int? a, int? b) {
    if (a == null || b == null) return null;
    final s = a + b;
    return s > maxSafe ? null : s;
  }

  /// `a * b`; null if one of them is null or the product exceeds [maxSafe].
  static int? mul(int? a, int? b) {
    if (a == null || b == null) return null;
    if (a == 0 || b == 0) return 0;
    if (a > maxSafe ~/ b) return null;
    return a * b;
  }

  /// `bool`.
  static const int boolean = 1;

  /// `uint_X`.
  static int uint(int bits) => bits;

  /// `int_X` (sign bit + magnitude).
  static int sint(int bits) => bits + 1;

  /// `uint_XdY`: `dynmax(X, Y)`.
  static int dynUint(int maxBits, int chunkBits) => _dynMax(maxBits, chunkBits);

  /// `int_XdY`: `1 + dynmax(X, Y)`.
  static int dynInt(int maxBits, int chunkBits) =>
      1 + _dynMax(maxBits, chunkBits);

  /// `float`.
  static const int float32 = 32;

  /// `double`.
  static const int float64 = 64;

  /// `fp_N[min, max]`.
  static int fixedPoint(int bits) => bits;

  /// `astr_dY[M]`: `dyn(M, 32, Y) + 8 * M`; null (unbounded) without `M`.
  static int? astr(int chunkBits, int? maxLength) =>
      _bytes(chunkBits, maxLength);

  /// `binary_dY[M]`: `dyn(M, 32, Y) + 8 * M`; null (unbounded) without `M`.
  static int? binary(int chunkBits, int? maxLength) =>
      _bytes(chunkBits, maxLength);

  static int? _bytes(int chunkBits, int? maxLength) =>
      dynArray(chunkBits, maxLength, 8);

  /// `repeated_dyn_Y[M] T`: `dyn(M, 32, Y) + M * max_bits(T)`; null without
  /// `M` or if [elementMaxBits] is null.
  static int? dynArray(int chunkBits, int? maxLength, int? elementMaxBits) {
    if (maxLength == null) return null;
    if (maxLength < 1) {
      throw ArgumentError.value(maxLength, 'maxLength', 'must be >= 1');
    }
    return add(dyn(maxLength, 32, chunkBits), mul(maxLength, elementMaxBits));
  }

  /// `optional T`: `1 + max_bits(T)`.
  static int? optional(int? valueMaxBits) => add(1, valueMaxBits);

  /// `repeated_fixed_N T`: `N * max_bits(T)`.
  static int? fixedArray(int length, int? elementMaxBits) =>
      mul(length, elementMaxBits);

  /// `enum E(Y)` with [count] values: `dyn(count - 1, 32, Y)`.
  static int enumeration(int count, int chunkBits) {
    if (count < 1) throw ArgumentError.value(count, 'count', 'must be >= 1');
    return dyn(count - 1, 32, chunkBits);
  }

  /// `oneof name(Y)`: `dyn(k - 1, 32, Y) + max over the k variants of
  /// max_bits(variant)`; null if a variant is unbounded.
  static int? oneOf(int chunkBits, Iterable<int?> variantMaxBits) {
    var k = 0;
    var m = 0;
    var unbounded = false;
    for (final v in variantMaxBits) {
      k++;
      if (v == null) {
        unbounded = true;
      } else if (v > m) {
        m = v;
      }
    }
    if (k == 0) throw ArgumentError('oneof needs at least one variant');
    final index = dyn(k - 1, 32, chunkBits);
    return unbounded ? null : add(index, m);
  }

  /// `msg`: sum of the attributes' max_bits; null if one is unbounded.
  static int? message(Iterable<int?> attributeMaxBits) {
    int? s = 0;
    for (final v in attributeMaxBits) {
      s = add(s, v);
    }
    return s;
  }
}
