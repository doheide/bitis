import 'dart:typed_data';

import 'bit_reader.dart';
import 'bit_writer.dart';
import 'errors.dart';

/// Version of this runtime package.
const String bitisRuntimeVersion = '0.11.0';

/// A bitis message (spec 2.15). Generated message classes implement this
/// and additionally provide `static Foo bitisRead(BitReader r)`.
abstract class BitisMessage {
  /// Writes all attributes in declaration order.
  void bitisWrite(BitWriter w);
}

/// Result of [bitisEncodeWithSize].
final class BitisEncoded {
  /// Creates an encoding result.
  const BitisEncoded(this.bytes, this.totalBits);

  /// The encoded bytes (zero padded to a byte boundary).
  final Uint8List bytes;

  /// Number of significant bits.
  final int totalBits;

  /// `ceil(totalBits / 8)` == `bytes.length`.
  int get totalBytes => bytes.length;
}

/// Result of [bitisDecode].
final class BitisDecoded<T> {
  /// Creates a decoding result.
  const BitisDecoded(this.value, this.totalBits);

  /// The decoded value.
  final T value;

  /// Number of bits consumed.
  final int totalBits;

  /// Number of bytes consumed: `ceil(totalBits / 8)`.
  int get totalBytes => (totalBits + 7) ~/ 8;
}

/// Encodes [message] into bytes.
Uint8List bitisEncode(BitisMessage message) {
  final w = BitWriter();
  message.bitisWrite(w);
  return w.toBytes();
}

/// Encodes [message] and also returns the number of significant bits.
BitisEncoded bitisEncodeWithSize(BitisMessage message) {
  final w = BitWriter();
  message.bitisWrite(w);
  return BitisEncoded(w.toBytes(), w.bitLength);
}

/// Decodes a value from the start of [data] using [read] (typically a
/// generated `Foo.bitisRead`). Trailing bytes are ignored; the consumed size
/// is returned in the result.
///
/// Throws [BitisDecodeException] on invalid or truncated input.
BitisDecoded<T> bitisDecode<T>(List<int> data, BitisReadFn<T> read) {
  final r = BitReader(data);
  final v = read(r);
  return BitisDecoded<T>(v, r.bitPosition);
}

/// Like [bitisDecode] but fails with [BitisDecodeErrorKind.trailingData] if
/// [data] is longer than `ceil(totalBits / 8)` bytes. Padding bits are not
/// checked.
T bitisDecodeExact<T>(List<int> data, BitisReadFn<T> read) {
  final r = BitReader(data);
  final v = read(r);
  if (data.length > r.bytePosition) {
    throw BitisDecodeException(
      BitisDecodeErrorKind.trailingData,
      '${data.length - r.bytePosition} trailing bytes',
      r.bitPosition,
    );
  }
  return v;
}
