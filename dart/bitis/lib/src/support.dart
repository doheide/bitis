/// Equality and hash helpers used by generated `==` / `hashCode`.
///
/// Lists (including `Uint8List`) are compared element by element; fixed point
/// values are compared on their raw wire value (spec 2.7), which generated
/// code passes as `equals` / `hashOf` (e.g. `latFpType.sameEncoding`).
abstract final class BitisEq {
  /// True if [a] and [b] have the same length and equal elements (compared
  /// with [equals] if given, otherwise with `==`).
  static bool listEquals<T>(
    List<T> a,
    List<T> b, [
    bool Function(T a, T b)? equals,
  ]) {
    if (identical(a, b)) return true;
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (equals == null ? a[i] != b[i] : !equals(a[i], b[i])) return false;
    }
    return true;
  }

  /// Hash code of a list consistent with [listEquals]; [hashOf] maps an
  /// element to the value that is hashed (default: the element itself).
  static int listHash<T>(List<T> values, [Object? Function(T v)? hashOf]) =>
      Object.hashAll(hashOf == null ? values : values.map(hashOf));

  /// Equality of two optional values: both null, or both non-null and
  /// equal according to [equals].
  static bool optionalEquals<T extends Object>(
    T? a,
    T? b,
    bool Function(T a, T b) equals,
  ) {
    if (a == null || b == null) return a == null && b == null;
    return equals(a, b);
  }

  /// Value hashed for an optional value consistent with [optionalEquals]:
  /// null, or [hashOf] of the value.
  static Object? optionalHash<T extends Object>(
    T? value,
    Object? Function(T v) hashOf,
  ) =>
      value == null ? null : hashOf(value);
}

/// Default value of the parameters of generated `copyWith` methods for
/// optional attributes: "keep the current value" (so that `null` can be
/// passed to clear the attribute).
const Object bitisKeep = _BitisKeep();

final class _BitisKeep {
  const _BitisKeep();

  @override
  String toString() => 'bitisKeep';
}

/// Returns [current] if [value] is [bitisKeep], otherwise [value] (which must
/// be a `T?`). Used by generated `copyWith` methods.
T? bitisKeepOr<T>(Object? value, T? current) =>
    identical(value, bitisKeep) ? current : value as T?;
