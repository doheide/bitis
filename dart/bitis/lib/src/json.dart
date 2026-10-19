import 'dart:convert';
import 'dart:typed_data';

import 'package:fixnum/fixnum.dart';

import 'fixed_point.dart';

/// Helpers for generated `toJson()` / `fromJson()` code.
///
/// JSON mapping (agreed with the app team):
/// * messages: JSON objects; `optional` fields are omitted when absent,
/// * enums: the variant name as a string ([enumFromJson]),
/// * oneof: `{"variant": value}` ([oneOfToJson] / [oneOfFromJson]),
/// * integers: numbers; [Int64] / [BigInt] values outside ±(2^53 - 1) as
///   decimal strings ([int64ToJson], [bigIntToJson]),
/// * fixed point: a number in physical units, underflow/overflow as
///   `{"state": "underflow"}` / `{"state": "overflow"}`,
/// * binary: standard base64 with padding,
/// * astr: string, ASCII only,
/// * float/double: numbers; NaN and ±infinity (not representable in JSON) as
///   the strings `"NaN"`, `"Infinity"`, `"-Infinity"`.
///
/// All `...FromJson` functions throw [FormatException] for values of the
/// wrong JSON type. Range checks against the field width happen when the
/// message is encoded (or in generated validation code), not here.
abstract final class BitisJson {
  /// JavaScript `Number.MAX_SAFE_INTEGER` (2^53 - 1).
  static const int maxSafeInteger = 9007199254740991;

  // Under dart2js ints are doubles; a large JSON number may not be exact.
  static const bool _jsNumbers = identical(1, 1.0);

  static final BigInt _maxSafeBig = BigInt.from(maxSafeInteger);
  static final BigInt _int64Min = BigInt.parse('-9223372036854775808');
  static final BigInt _int64Max = BigInt.parse('9223372036854775807');
  static final RegExp _decimal = RegExp(r'^-?[0-9]+$');

  static Never _bad(String expected, Object? json) => throw FormatException(
      'expected $expected, got ${json.runtimeType}', json);

  /// Parses an exact integer from a JSON number or decimal string.
  static BigInt _integer(Object? json, String what) {
    if (json is String) {
      if (!_decimal.hasMatch(json)) _bad('$what (decimal string)', json);
      return BigInt.parse(json);
    }
    if (json is num) {
      if (json is! int) {
        // Native platforms: an integral double like 3.0.
        if (!json.isFinite || json != json.truncateToDouble()) {
          _bad('$what (integral number)', json);
        }
        if (json.abs() > maxSafeInteger) {
          _bad('$what (number within ±2^53-1, use a string)', json);
        }
        return BigInt.from(json);
      }
      if (_jsNumbers && json.abs() > maxSafeInteger) {
        _bad('$what (number within ±2^53-1, use a string)', json);
      }
      return BigInt.from(json);
    }
    _bad(what, json);
  }

  // ------------------------------------------------------------- integers

  /// JSON value of an `int` field (widths <= 32 bits): the number itself.
  static int intToJson(int value) => value;

  /// Parses an `int` field: integral number or decimal string, within
  /// ±(2^53 - 1).
  static int intFromJson(Object? json) {
    final b = _integer(json, 'integer');
    if (b.abs() > _maxSafeBig) _bad('integer within ±2^53-1', json);
    return b.toInt();
  }

  /// JSON value of an [Int64] field: a number if `|value| <= 2^53 - 1`,
  /// otherwise a decimal string.
  static Object int64ToJson(Int64 value) {
    if (value <= Int64(maxSafeInteger) && value >= Int64(-maxSafeInteger)) {
      return value.toInt();
    }
    return value.toString();
  }

  /// Parses an [Int64] field from a number or decimal string; throws
  /// [FormatException] outside the signed 64-bit range.
  static Int64 int64FromJson(Object? json) {
    final b = _integer(json, 'integer');
    if (b < _int64Min || b > _int64Max) _bad('64-bit signed integer', json);
    return Int64.parseInt(b.toString());
  }

  /// JSON value of a [BigInt] field (`uint_64`): a number if
  /// `|value| <= 2^53 - 1`, otherwise a decimal string.
  static Object bigIntToJson(BigInt value) =>
      value.abs() <= _maxSafeBig ? value.toInt() : value.toString();

  /// Parses a [BigInt] field from a number or decimal string.
  static BigInt bigIntFromJson(Object? json) => _integer(json, 'integer');

  // --------------------------------------------------------------- others

  /// Parses a `bool` field.
  static bool boolFromJson(Object? json) =>
      json is bool ? json : _bad('bool', json);

  /// JSON value of a `float`/`double` field (non-finite values as strings).
  static Object doubleToJson(double value) {
    if (value.isFinite) return value;
    if (value.isNaN) return 'NaN';
    return value > 0 ? 'Infinity' : '-Infinity';
  }

  /// Parses a `float`/`double` field from a number or one of `"NaN"`,
  /// `"Infinity"`, `"-Infinity"`.
  static double doubleFromJson(Object? json) {
    if (json is num) return json.toDouble();
    switch (json) {
      case 'NaN':
        return double.nan;
      case 'Infinity':
        return double.infinity;
      case '-Infinity':
        return double.negativeInfinity;
    }
    _bad('number', json);
  }

  /// JSON value of a fixed point field: the physical value, or
  /// `{"state": "underflow"}` / `{"state": "overflow"}`. A non-finite value
  /// is mapped like the encoder does (NaN and -inf underflow, +inf overflow).
  static Object fixedPointToJson(FixedPoint value) {
    switch (value.state) {
      case FixedPointState.underflow:
        return const {'state': 'underflow'};
      case FixedPointState.overflow:
        return const {'state': 'overflow'};
      case FixedPointState.value:
        final v = value.value;
        if (v.isFinite) return v;
        return v == double.infinity
            ? const {'state': 'overflow'}
            : const {'state': 'underflow'};
    }
  }

  /// Parses a fixed point field. If [type] is given, the value is classified
  /// and quantized with [FixedPointType.fromDouble].
  static FixedPoint fixedPointFromJson(Object? json, [FixedPointType? type]) {
    if (json is num) {
      final v = json.toDouble();
      return type == null ? FixedPoint.value(v) : type.fromDouble(v);
    }
    if (json is Map && json.length == 1) {
      switch (json['state']) {
        case 'underflow':
          return const FixedPoint.underflow();
        case 'overflow':
          return const FixedPoint.overflow();
      }
    }
    _bad('number or {"state": "underflow"|"overflow"}', json);
  }

  /// JSON value of a `binary` field: standard base64.
  static String binaryToJson(List<int> value) => base64Encode(value);

  /// Parses a `binary` field from base64 (standard or URL-safe alphabet).
  static Uint8List binaryFromJson(Object? json) {
    if (json is! String) _bad('base64 string', json);
    return base64Decode(json);
  }

  /// True if every character of [value] is ASCII (< 0x80).
  static bool isAscii(String value) {
    for (final c in value.codeUnits) {
      if (c >= 0x80) return false;
    }
    return true;
  }

  /// JSON value of an `astr` field (the string itself).
  static String astrToJson(String value) => value;

  /// Parses an `astr` field; non-ASCII strings are rejected.
  static String astrFromJson(Object? json) {
    if (json is! String) _bad('string', json);
    if (!isAscii(json)) throw FormatException('astr must be ASCII', json);
    return json;
  }

  /// Looks up an enum variant by its JSON name.
  static T enumFromJson<T>(Object? json, Map<String, T> byName) {
    if (json is! String) _bad('enum name string', json);
    final v = byName[json];
    if (v == null) throw FormatException('unknown enum value', json);
    return v;
  }

  /// JSON value of a oneof: `{variant: value}`.
  static Map<String, Object?> oneOfToJson(String variant, Object? value) =>
      {variant: value};

  /// Splits a oneof JSON object `{variant: value}` (exactly one key).
  static (String, Object?) oneOfFromJson(Object? json) {
    if (json is! Map || json.length != 1) _bad('object with one key', json);
    final MapEntry(:key, :value) = json.entries.single;
    if (key is! String) _bad('string key', key);
    return (key, value);
  }

  /// Casts a JSON object (as produced by `jsonDecode`).
  static Map<String, Object?> objectFromJson(Object? json) {
    if (json is! Map) _bad('object', json);
    return json.cast<String, Object?>();
  }

  /// Casts a JSON array; if [length] is given the array must have exactly
  /// that many elements (fixed arrays).
  static List<Object?> listFromJson(Object? json, [int? length]) {
    if (json is! List) _bad('array', json);
    if (length != null && json.length != length) {
      throw FormatException('expected $length elements', json);
    }
    return json.cast<Object?>();
  }
}
