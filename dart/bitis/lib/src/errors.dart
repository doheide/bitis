/// Reason why decoding a bitis bit stream failed.
///
/// The names are identical in every bitis runtime (see `docs/WIRE_FORMAT.md`,
/// section 3).
enum BitisDecodeErrorKind {
  /// The input ended before the message was complete. In a stream this means
  /// "wait for more data".
  unexpectedEnd,

  /// A field value is outside its valid range, or an `astr` contains a byte
  /// >= 0x80.
  invalidValue,

  /// Unknown enum or oneof index.
  invalidDiscriminant,

  /// Strict decoding only: the input is longer than the message needs.
  trailingData,
}

/// Reason why encoding a value failed.
enum BitisEncodeErrorKind {
  /// A value does not fit its declared width / base type, a string is not
  /// ASCII, a fixed array has the wrong length or a count exceeds 2^32 - 1.
  valueOutOfRange,
}

/// Thrown by [BitReader] methods and the decode helpers when the input is not
/// a valid encoding.
///
/// Decoding untrusted input never throws anything else (programming errors
/// such as an invalid bit width passed by generated code throw
/// [ArgumentError]).
final class BitisDecodeException implements Exception {
  /// Creates a decode exception.
  const BitisDecodeException(this.kind, [this.message = '', this.bitPosition]);

  /// Machine readable reason.
  final BitisDecodeErrorKind kind;

  /// Human readable details (may be empty).
  final String message;

  /// Bit position in the input at which the error was detected, if known.
  final int? bitPosition;

  @override
  String toString() {
    final pos = bitPosition == null ? '' : ' at bit $bitPosition';
    final msg = message.isEmpty ? '' : ': $message';
    return 'BitisDecodeException(${kind.name}$pos)$msg';
  }
}

/// Thrown by [BitWriter] methods when a value cannot be encoded.
final class BitisEncodeException implements Exception {
  /// Creates an encode exception.
  const BitisEncodeException(this.kind, [this.message = '']);

  /// Machine readable reason.
  final BitisEncodeErrorKind kind;

  /// Human readable details (may be empty).
  final String message;

  @override
  String toString() {
    final msg = message.isEmpty ? '' : ': $message';
    return 'BitisEncodeException(${kind.name})$msg';
  }
}
